
use authorizer::Auth;
use crate::{
    aws::appsync,
};

pub async fn list(auth: &Auth, name: &str) {
    let client = appsync::make_client(auth).await;
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
