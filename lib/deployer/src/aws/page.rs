use composer::{
    Page,
    page::BucketPolicy,
};
use kit as u;
use kit::*;
use provider::{
    Auth,
    aws::{
        acm,
        cloudfront,
        route53,
        s3,
        ssm,
    },
};
use std::collections::HashMap;

async fn build_page(auth: &Auth, dir: &str, name: &str, command: &Option<String>, config_template: &Option<String>) {
    match command {
        Some(c) => {
            builder::page::build(auth, dir, name, &c, config_template).await;
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
    //let auth = provider::init_centralized_auth(auth).await;
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

        let out_file = if path == "index.html" {
            u::sh("mkdir -p dist", dir);
            u::sh(&format!("cp {} dist/{}", path, path), dir);
            format!("{}/dist/{}", dir, path)
        } else {
            format!("{}_tmp", &p)
        };
        u::write_str(&out_file, &rs);
    } else {
        println!("Config template {} does not exist", path);
    }
}

async fn find_or_create_cert(auth: &Auth, domain: &str, token: &str) -> String {
    let client = acm::make_client(auth).await;

    let maybe_cert = acm::find_cert(&client, domain).await;
    let cert_arn = if let Some(arn) = maybe_cert {
        tracing::debug!("Cert already exists {}", &arn);
        arn
    } else {
        println!("Creating cert {}", domain);
        acm::request_cert(&client, domain, token).await
    };
    u::sleep(1000);
    if !acm::is_cert_issued(&client, &cert_arn).await {
        u::sleep(10000);
        let validation_records = acm::get_domain_validation_records(&client, &cert_arn).await;
        let route53_client = route53::make_client(auth).await;
        for rec in validation_records {
            route53::create_record_set(
                &route53_client,
                domain,
                &rec.name,
                &rec.r#type.as_str(),
                &rec.value,
            )
            .await;
        }
        acm::wait_until_validated(&client, &cert_arn).await;
    } else {
        println!("Checking cert status: Issued");
    }
    cert_arn
}

fn find_domain(
    domains: &HashMap<String, HashMap<String, String>>,
    env: &str,
    sandbox: &str,
) -> Option<String> {
    match domains.get(env) {
        Some(e) => e.get(sandbox).cloned(),
        None => match domains.get("default") {
            Some(d) => d.get(sandbox).cloned(),
            None => None,
        },
    }
}

async fn update_bucket_policy(
    auth: &Auth,
    bucket: &str,
    bucket_policy: &BucketPolicy,
    dist_id: &str,
) {
    tracing::debug!("Updating bucket policy");
    let s3_client = s3::make_client(auth).await;
    let existing_policy = s3::get_bucket_policy(&s3_client, bucket).await;
    let policy = augment_policy(existing_policy, bucket_policy.clone(), dist_id);
    s3::update_bucket_policy(&s3_client, bucket, &policy).await;
}

async fn update_dns_record(auth: &Auth, domain: &str, dist_id: &str, cname: &str) {
    tracing::debug!("Associating domain {} with {}", domain, &dist_id);
    let rclient = route53::make_client(auth).await;
    route53::create_record_set(&rclient, domain, domain, "CNAME", cname).await;
}

async fn build(auth: &Auth, name: &str, page: &Page, config: &HashMap<String, String>) {
    let Page {
        dir,
        build,
        config_template,
        ..
    } = page;
    if let Some(path) = config_template {
        println!("Rendering config: {}", &path);
        render_config_template(auth, dir, &path, config).await;
    }
    println!("Building page {} ({})", name, dir);
    build_page(auth, dir, name, build, config_template).await;
}

async fn build_and_upload(auth: &Auth, name: &str, page: &Page, config: &HashMap<String, String>) {
    let Page {
        bucket,
        bucket_prefix,
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
    println!("Building page {} ({})", name, dir);
    build_page(auth, dir, name, build, config_template).await;

    if !u::path_exists(&u::pwd(), dist) {
        tracing::debug!("Dist directory not found, aborting");
        return;
    }
    let s3_client = s3::make_client(auth).await;

    s3::find_or_create_bucket(&s3_client, bucket).await;

    if bucket.is_empty() {
        panic!("Bucket not configured. Set TC_PAGES_BUCKET, in config or in topology")
    }

    println!(
        "Uploading code from {} to s3://{}/{}",
        dist, bucket, bucket_prefix
    );

    s3::upload_dir(&s3_client, dist, bucket, bucket_prefix).await;
}

fn as_function_arns(auth: &Auth, functions: &HashMap<String, String>) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    for (name, _) in functions {
        xs.push(auth.cloudfront_function_arn(&name));
    }
    xs
}

async fn create_or_update_distribution(
    auth: &Auth,
    name: &str,
    page: &Page,
    sandbox: &str,
    maybe_domain: Option<String>,
) -> (String, String) {
    let Page {
        fqn,
        origin_paths,
        origin_domain,
        caller_ref,
        default_root_object,
        functions,
        ..
    } = page;

    let client = cloudfront::make_client(auth).await;

    tracing::debug!("Configuring page {} - setting OAC ", name);
    let oac_id = cloudfront::find_or_create_oac(&client, origin_domain).await;

    tracing::debug!("Configuring page {} - setting cache policy ", name);
    let cache_policy_id = cloudfront::find_or_create_cache_policy(&client, caller_ref).await;

    let maybe_cert_arn = if let Some(domain) = &maybe_domain {
        let idempotency_token = sandbox;
        let arn = find_or_create_cert(auth, domain, idempotency_token).await;
        Some(arn)
    } else {
        None
    };

    let dist_config = cloudfront::make_dist_config(
        fqn,
        default_root_object,
        caller_ref,
        origin_domain,
        origin_paths.clone(),
        maybe_domain.clone(),
        maybe_cert_arn,
        &oac_id,
        &cache_policy_id,
        as_function_arns(auth, functions),
    );

    for (name, handler) in functions {
        println!("Configuring page: creating function {}", name);
        cloudfront::create_or_update_function(&client, name, handler).await;
    }

    println!("Configuring page {} - creating distribution", name);
    let dist_id = cloudfront::create_or_update_distribution(&client, fqn, dist_config).await;

    cloudfront::wait_until_updated(&client, &dist_id).await;

    let cname = cloudfront::get_cname(&client, &dist_id).await;

    tracing::debug!("Configuring page {} - invalidating cache", name);
    cloudfront::create_invalidation(&client, &dist_id).await;
    (dist_id, cname)
}

async fn create_page(
    auth: &Auth,
    name: &str,
    page: &Page,
    config: &HashMap<String, String>,
    sandbox: &str,
) {
    let Page {
        bucket,
        bucket_policy,
        domains,
        ..
    } = page;

    let maybe_domain = find_domain(domains, &auth.name, sandbox);

    println!("Building page");
    build_and_upload(auth, name, page, config).await;
    let (dist_id, cname) =
        create_or_update_distribution(auth, name, page, sandbox, maybe_domain.clone()).await;

    if let Some(domain) = &maybe_domain {
        update_dns_record(auth, domain, &dist_id, &cname).await;
        println!("url - https://{}", domain);
    } else {
        println!("url - https://{}", &cname);
    }

    update_bucket_policy(auth, bucket, bucket_policy, &dist_id).await;
}

pub async fn create(
    auth: &Auth,
    pages: &HashMap<String, Page>,
    config: &HashMap<String, String>,
    sandbox: &str,
) {
    for (name, page) in pages {
        if page.skip_deploy {
            println!("Skipping page deploy {}", &name);
            build(auth, &name, &page, config).await;
        } else {
            create_page(auth, &name, &page, config, sandbox).await;
        }
    }
}

async fn update_code(auth: &Auth, pages: &HashMap<String, Page>, config: &HashMap<String, String>) {
    for (name, page) in pages {
        let Page { namespace, .. } = page;

        println!("Building page");
        build_and_upload(auth, name, page, config).await;

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
            println!("Config: ");
            for (k, v) in config {
                println!("{}={}", k, v);
            }
            render_config_template(auth, dir, &path, config).await;
        }
    }
}

async fn update_domains(
    auth: &Auth,
    pages: &HashMap<String, Page>,
    _config: &HashMap<String, String>,
    sandbox: &str,
) {
    for (name, page) in pages {
        let Page { domains, .. } = page;

        let maybe_domain = find_domain(domains, &auth.name, sandbox);
        let (dist_id, cname) =
            create_or_update_distribution(auth, name, page, sandbox, maybe_domain.clone()).await;

        if let Some(domain) = &maybe_domain {
            update_dns_record(auth, domain, &dist_id, &cname).await;
            println!("url - https://{}", domain);
        } else {
            println!("url - https://{}", &cname);
        }
    }
}

async fn update_functions(auth: &Auth, pages: &HashMap<String, Page>) {
    let client = cloudfront::make_client(auth).await;
    for (_, page) in pages {
        for (name, handler) in &page.functions {
            println!("Configuring page {} - creating function", &name);
            cloudfront::create_or_update_function(&client, &name, &handler).await;
        }
    }
}

pub async fn update(
    auth: &Auth,
    pages: &HashMap<String, Page>,
    component: &str,
    config: &HashMap<String, String>,
    sandbox: &str,
) {
    match component {
        "code" => update_code(auth, pages, config).await,
        "config" => update_config(auth, pages, config).await,
        "domains" => update_domains(auth, pages, config, sandbox).await,
        "functions" => update_functions(auth, pages).await,
        "build" => {
            for (name, page) in pages {
                build_page(auth, &page.dir, name, &page.build, &page.config_template).await;
            }
        }
        _ => {
            if let Some(page) = pages.get(component) {
                create_page(auth, component, page, config, sandbox).await;
            } else {
                update_code(auth, pages, config).await;
            }
        }
    }
}

async fn delete_page(auth: &Auth, page: &Page) {
    let client = cloudfront::make_client(auth).await;
    cloudfront::delete_distribution(&client, &page.fqn).await;
}

pub async fn delete(auth: &Auth, pages: &HashMap<String, Page>) {
    for (name, page) in pages {
        println!("Deleting page {}", &name);
        delete_page(auth, page).await;
    }
}

pub async fn create_dry_run(pages: &HashMap<String, Page>) {
    for (name, _page) in pages {
        println!("Creating page {}", &name);
    }
}
