use crate::aws::{
    cloudfront,
    s3,
};
use compiler::Page;
use authorizer::Auth;
use kit::*;
use std::collections::HashMap;


fn build_page(dir: &str, dist: &str, command: &str) {
    sh(command, dir);
}

async fn create_page(auth: &Auth, page: &Page) {

    let Page { bucket, bucket_policy,
               origin_path,
               origin_domain, caller_ref,
               dist, dir, build, domains, paths, .. } = page;

    // TODO build
    build_page(dir, dist, build);

    let s3_client = s3::make_client(auth).await;

    // create s3 bucket
    s3::find_or_create_bucket(&s3_client, bucket).await;

    // upload dist dir
    s3::upload_dir(&s3_client, dist, bucket).await;

    let client = cloudfront::make_client(auth).await;
    // origin access identity
    let oai_id = create_oai(&client, caller_ref).await;

    // origin access control
    let oac_id = create_oac(&client, caller_ref).await;

    let dist_config = cloudfront::make_dist_config(
        caller_ref,
        bucket,
        oai_id,
        paths,
    )

    cloudfront::create_or_update_distribution(
        &client,
        origin_domain,
        origin_paths,
        bucket,
        &oai_id,

    ).await;

    // TODO use oai_id in bucket_policy.

    // render bucket policy

    // set bucket permissions
    s3::update_bucket_policy(&s3_client, &page.bucket, &page.bucket_policy).await;



}


pub async fn create(auth: &Auth, pages: &HashMap<String, Page>) {

    for page in pages {
        create_page(auth, page).await

    }

}
