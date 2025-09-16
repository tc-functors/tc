use composer::{
    Page,
    topology::page::BucketPolicy,
};
use kit as u;
use kit::*;
use provider::{
    Auth,
    aws::{
        cloudfront,
        s3,
        ssm,
    },
};
use std::collections::HashMap;

fn build_page(dir: &str, name: &str, command: &Option<String>, config_template: &Option<String>) {
    match command {
        Some(c) => {
            builder::page::build(dir, name, &c, config_template);
        }
        None => (),
    }
}

// policy
fn render(s: &str, id: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("lazy_id", id);
    u::stencil(s, table)
}

fn augment_policy(
    existing_policy: Option<String>,
    given_policy: BucketPolicy,
    dist_id: &str,
) -> String {
    let policy = match existing_policy {
        Some(e) => given_policy.add_statement(&e),
        None => given_policy,
    };
    let policy_str = serde_json::to_string(&policy).unwrap();
    render(&policy_str, dist_id)
}

async fn resolve_vars(auth: &Auth, keys: Vec<String>) -> HashMap<String, String> {
    let auth = provider::init_centralized_auth(auth).await;
    let client = ssm::make_client(&auth).await;

    let mut h: HashMap<String, String> = HashMap::new();
    for k in keys {
        if k.starts_with("ssm:/") {
            tracing::debug!("Resolving SSM var {}", &k);
            let key = kit::split_last(&k, ":");
            let val = ssm::get(client.clone(), &key).await.unwrap();
            if !val.is_empty() {
                h.insert(s!(k), val);
            }
        }
    }
    h
}

async fn resolve_entities(auth: &Auth, keys: Vec<String>) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    for k in keys {
        if k.starts_with("mutation:/") {
            let client = provider::aws::appsync::make_client(auth).await;
            tracing::debug!("Resolving Mutations var {}", &k);
            let key = kit::split_last(&k, ":/");
            let api = provider::aws::appsync::find_graphql_api(&client, &key).await;
            if let Some(a) = api {
                h.insert(s!(k), a.https.clone());
            }
        }
    }
    h
}

async fn render_config_template(
    auth: &Auth,
    dir: &str,
    path: &str,
    config: &HashMap<String, String>,
) {
    let p = format!("{}/{}", dir, path);
    if u::file_exists(&p) {

        let s = u::slurp(&p);
        let mut table: HashMap<&str, &str> = HashMap::new();
        for (k, v) in config {
            table.insert(&k, &v);
        }
        let mut rs = u::stencil(&s, table);

        // resolve ssm keys
        let matches = u::find_matches(&rs, r"ssm:/([^/]+)/(.*?([^/]+)/?)(\r\n|\r|\n)");
        let resolved = resolve_vars(auth, matches).await;

        for (k, v) in resolved {
            if let Some(m) = rs.find(&k) {
                let n = m + k.len();
                rs.replace_range(m..n, &v);
            }
        }

        let entity_matches = u::find_matches(&rs, r"mutation:/(.*?([^/]+)/?)(\r\n|\r|\n)");
        let resolved = resolve_entities(auth, entity_matches).await;

        for (k, v) in resolved {
            if let Some(m) = rs.find(&k) {
                let n = m + k.len();
                rs.replace_range(m..n, &v);
            }
        }

        let out_file = format!("{}_tmp", &p);
        u::write_str(&out_file, &rs);
    } else {
        println!("Config template {} does not exist", path);
    }
}

async fn create_page(auth: &Auth, name: &str, page: &Page, config: &HashMap<String, String>) {
    let Page {
        bucket,
        bucket_policy,
        bucket_prefix,
        origin_paths,
        namespace,
        origin_domain,
        caller_ref,
        dist,
        dir,
        build,
        domains,
        default_root_object,
        config_template,
        ..
    } = page;

    if let Some(path) = config_template {
        println!("Rendering config: {}", &path);
        render_config_template(auth, dir, &path, config).await;
    }

    println!("Building page {} ({})", name, dir);
    build_page(dir, name, build, config_template);

    if !u::path_exists(&u::pwd(), dist) {
        tracing::debug!("Dist directory not found, aborting");
        return;
    }

    println!("Creating page {}", name);

    let s3_client = s3::make_client(auth).await;

    s3::find_or_create_bucket(&s3_client, bucket).await;

    tracing::debug!(
        "Uploading code from {} to s3://{}/{}",
        dist,
        bucket,
        bucket_prefix
    );
    s3::upload_dir(&s3_client, dist, bucket, bucket_prefix).await;

    let client = cloudfront::make_client(auth).await;

    tracing::debug!("Configuring page {} - setting OAC ", name);
    let oac_id = cloudfront::find_or_create_oac(&client, origin_domain).await;

    tracing::debug!("Configuring page {} - setting cache policy ", name);
    let cache_policy_id = cloudfront::find_or_create_cache_policy(&client, caller_ref).await;

    let dist_config = cloudfront::make_dist_config(
        namespace,
        default_root_object,
        caller_ref,
        origin_domain,
        origin_paths.clone(),
        domains.clone(),
        &oac_id,
        &cache_policy_id,
    );

    tracing::debug!("Configuring page {} - creating distribution", name);
    let dist_id = cloudfront::create_or_update_distribution(&client, namespace, dist_config).await;

    let existing_policy = s3::get_bucket_policy(&s3_client, bucket).await;
    let policy = augment_policy(existing_policy, bucket_policy.clone(), &dist_id);
    s3::update_bucket_policy(&s3_client, bucket, &policy).await;

    tracing::debug!("Configuring page {} - invalidating cache", name);
    cloudfront::create_invalidation(&client, &dist_id).await;

    let url = cloudfront::get_url(&client, &dist_id).await;
    println!("url - https://{}", url);
}

pub async fn create(auth: &Auth, pages: &HashMap<String, Page>, config: &HashMap<String, String>) {
    for (name, page) in pages {
        create_page(auth, &name, &page, config).await
    }
}

async fn update_code(auth: &Auth, pages: &HashMap<String, Page>, config: &HashMap<String, String>) {
    for (name, page) in pages {
        let Page {
            bucket,
            bucket_prefix,
            namespace,
            dist,
            dir,
            build,
            config_template,
            ..
        } = page;

        if let Some(path) = config_template {
            println!("Rendering config: {}", &path);
            render_config_template(auth, dir, &path, config).await;
        }

        println!("Building page {}", name);
        build_page(dir, name, build, config_template);

        if !u::path_exists(&u::pwd(), dist) {
            println!("Dist directory not found, aborting");
            return;
        }

        let s3_client = s3::make_client(auth).await;

        s3::find_or_create_bucket(&s3_client, bucket).await;

        println!(
            "Uploading code from {} to s3://{}/{}",
            dist, bucket, bucket_prefix
        );
        s3::upload_dir(&s3_client, dist, bucket, bucket_prefix).await;

        let client = cloudfront::make_client(auth).await;
        println!("Configuring page {} - invalidating cache", name);
        let maybe_dist_id = cloudfront::find_distribution(&client, namespace).await;
        if let Some((dist_id, _)) = maybe_dist_id {
            cloudfront::create_invalidation(&client, &dist_id).await;
        }
    }
}

pub async fn update_config(
    auth: &Auth,
    pages: &HashMap<String, Page>,
    config: &HashMap<String, String>,
) {
    for (_, page) in pages {
        let Page {
            dir,
            config_template,
            ..
        } = page;
        if let Some(path) = config_template {
            println!("Rendering config: {}", &path);
            render_config_template(auth, dir, &path, config).await;
        }
    }
}

pub async fn update(
    auth: &Auth,
    pages: &HashMap<String, Page>,
    component: &str,
    config: &HashMap<String, String>,
) {
    match component {
        "code" => update_code(auth, pages, config).await,
        "config" => update_config(auth, pages, config).await,
        "build" => {
            for (name, page) in pages {
                build_page(&page.dir, name, &page.build, &page.config_template);
            }
        }
        _ => {
            if let Some(page) = pages.get(component) {
                create_page(auth, component, page, config).await;
            } else {
                update_code(auth, pages, config).await;
            }
        }
    }
}

pub async fn delete(_auth: &Auth, _pages: &HashMap<String, Page>) {
    for (name, _page) in _pages {
        println!("Deleting page {}", &name);
    }
}
