use crate::aws::{
    cloudfront,
    s3,
};
use compiler::Page;
use authorizer::Auth;
use kit as u;
use std::collections::HashMap;

fn build_page(dir: &str, name: &str, command: &Option<String>) {
    match command {
        Some(c) => {
            builder::page::build(dir, name, &c);
        },
        None => ()
    }
}

fn render(s: &str, id: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("lazy_id", id);
    u::stencil(s, table)
}

async fn create_page(auth: &Auth, name: &str, page: &Page) {

    let Page { bucket, bucket_policy, bucket_prefix,
               origin_paths,
               origin_domain, caller_ref,
               dist, dir, build, domains, default_root_object, .. } = page;

    println!("Building page {}", name);
    build_page(dir, name, build);

    let s3_client = s3::make_client(auth).await;

    s3::find_or_create_bucket(&s3_client, bucket).await;

    println!("Uploading code from {} to s3://{}/{}", dist, bucket, bucket_prefix);
    s3::upload_dir(&s3_client, dist, bucket, bucket_prefix).await;

    let client = cloudfront::make_client(auth).await;

    println!("Configuring page {} - setting OAC ", name);
    let oac_id = cloudfront::find_or_create_oac(&client, origin_domain).await;

    println!("Configuring page {} - setting cache policy ", name);
    let cache_policy_id = cloudfront::find_or_create_cache_policy(&client, caller_ref).await;

    let dist_config = cloudfront::make_dist_config(
        default_root_object,
        caller_ref,
        origin_domain,
        origin_paths.clone(),
        domains.clone(),
        &oac_id,
        &cache_policy_id
    );

    println!("Configuring page {} - creating distribution", name);
    let dist_id = cloudfront::create_or_update_distribution(&client, origin_domain, dist_config).await;

    println!("Configuring page {} - updating bucket policy", name);
    let policy = render(bucket_policy, &dist_id);
    s3::update_bucket_policy(&s3_client, bucket, &policy).await;

    println!("Configuring page {} - invalidating cache", name);
    cloudfront::create_invalidation(&client, &dist_id).await;

    let url = cloudfront::get_url(&client, &dist_id).await;
    println!("url - https://{}", url);
}

pub async fn create(auth: &Auth, pages: &HashMap<String, Page>) {
    for (name, page) in pages {
        create_page(auth, &name, &page).await
    }
}

pub async fn update(_auth: &Auth, _pages: &HashMap<String, Page>, _c: &str) {

}

pub async fn delete(_auth: &Auth, _pages: &HashMap<String, Page>) {
    for (name, _page) in _pages {
        println!("Deleting page {}", &name);
    }
}
