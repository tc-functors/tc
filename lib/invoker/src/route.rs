use crate::aws::gateway;
use authorizer::Auth;
use kit as u;
use kit::*;
use std::collections::HashMap;
use serde_json::Value;

pub async fn request(
    auth: &Auth,
    api_name: &str,
    path: &str,
    method: &str
) -> Value {


    let client = gateway::make_client(auth).await;
    let maybe_api_id = gateway::find_api_id(&client, api_name).await;
    if let Some(api_id) = maybe_api_id {
        let endpoint = auth.api_endpoint(&api_id, "$default");
        let url = format!("{}{}", &endpoint, path);
        //println!("Invoking {}", &url);

        let mut h = HashMap::new();
        h.insert(s!("content-type"), s!("application/json"));
        h.insert(s!("accept"), s!("application/json"));
        h.insert(
            s!("user-agent"),
            s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
        );

        match method {
            "GET" => u::http_get(&url, h).await,
            "POST" => todo!(),
            &_ => todo!()
        }



    } else {
        panic!("No gateway found");
    }
}
