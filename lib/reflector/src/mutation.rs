use provider::Auth;
use provider::aws::appsync;

pub async fn introspect(auth: &Auth, name: &str) {
    let client = appsync::make_client(auth).await;
    if let Some(api_id) = appsync::find_api(&client, name).await {
        if let Some(s) = appsync::get_schema(&client, &api_id).await {
            println!("{}", &s);
        }
    }
}
