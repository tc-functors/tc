use crate::aws::{
    cloudfront,
    s3,
};
use compiler::Page;
use authorizer::Auth;
use kit::*;
use kit as u;
use std::collections::HashMap;

fn build_page(dir: &str, dist: &str, command: &str) {
    if !command.is_empty() {
        sh(command, dir);
    }
}

fn render(s: &str, id: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("oai_id", id);
    u::stencil(s, table)
}

async fn create_page(auth: &Auth, name: &str, page: &Page) {

    let Page { bucket, bucket_policy,
               origin_paths,
               origin_domain, caller_ref,
               dist, dir, build, domains, .. } = page;

    println!("Building page {}", name);
    build_page(dir, dist, build);

    let s3_client = s3::make_client(auth).await;

    s3::find_or_create_bucket(&s3_client, bucket).await;

    println!("Uploading code from {}", dist);
    s3::upload_dir(&s3_client, dist, bucket).await;

    let client = cloudfront::make_client(auth).await;

    println!("Updating page {} - OAI", name);
    let oai_id = cloudfront::create_oai(&client, caller_ref).await;

    println!("Updating page {} - OAC", name);
    let _oac_id = cloudfront::create_oac(&client, caller_ref).await;

    let dist_config = cloudfront::make_dist_config(
        caller_ref,
        origin_domain,
        origin_paths.clone(),
        domains.clone(),
        &oai_id
    );

    println!("Updating page {} - creating distribution {}", name, &origin_domain);
    cloudfront::create_or_update_distribution(&client, origin_domain, dist_config).await;

    let policy = render(bucket_policy, &oai_id);

    // set bucket permissions
    s3::update_bucket_policy(&s3_client, bucket, policy).await;
}

pub async fn create(auth: &Auth, pages: &HashMap<String, Page>) {
    for (name, page) in pages {
        create_page(auth, &name, &page).await
    }
}

pub async fn update(_auth: &Auth, _pages: &HashMap<String, Page>, _c: &str) {

}

pub async fn delete(auth: &Auth, pages: &HashMap<String, Page>) {
    for (name, _page) in pages {
        println!("Deleting page {}", &name);
    }
}
