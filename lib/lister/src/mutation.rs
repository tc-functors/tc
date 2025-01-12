use aws::appsync;
use aws::Env;

pub async fn list(env: &Env, name: &str) {
    let client = appsync::make_client(env).await;
    let api = appsync::find_api(&client, name).await;
    match api {
        Some(a) => {
            println!("id: {}", &a.id);
            println!("https: {}", &a.https);
            println!("wss: {}", &a.wss);
        }
        _ => (),
    }
}
