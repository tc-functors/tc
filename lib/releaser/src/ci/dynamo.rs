use crate::aws::dynamo;
use authorizer::Auth;
use kit as u;

// deprecated
pub async fn put_item(auth: &Auth, service: &str, version: &str, deploy_env: &str, dir: &str) {
    let client = dynamo::make_client(auth).await;
    let table = "service-version-inventory-tc";
    let deploy_time = u::utc_now();
    let deploy_user = u::env_var("CIRCLE_USERNAME", "default");
    let build_url = u::env_var("CIRCLE_BUILD_URL", "default");
    println!("Stashing {}  {}", service, version);
    let _res = client
        .put_item()
        .table_name(table)
        .item("service_name", dynamo::attr(service))
        .item("environment", dynamo::attr(deploy_env))
        .item("service_version", dynamo::attr(version))
        .item("deploy_time", dynamo::attr(&deploy_time))
        .item("deploy_owner", dynamo::attr(&deploy_user))
        .item("build_url", dynamo::attr(&build_url))
        .item("directory", dynamo::attr(dir))
        .send()
        .await;
}
