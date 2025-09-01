use crate::aws::lambda;
use authorizer::Auth;
use kit as u;
use kit::*;
use std::collections::HashMap;

pub async fn invoke(auth: &Auth, name: &str, payload: &str) {
    let client = lambda::make_client(auth).await;
    println!("Invoking function {}", name);
    let _ = lambda::invoke(client, name, payload).await;
}

pub async fn invoke_emulator(payload: &str) {
    let mut headers = HashMap::new();
    headers.insert(s!("content-type"), s!("application/json"));
    let url = "http://localhost:9000/2015-03-31/functions/function/invocations";
    let res = u::http_post(url, headers, payload.to_string()).await;
    println!("{:?}", res);
    let out = match res {
        Ok(r) => kit::pretty_json(&r),
        Err(_) => s!("Error invoking local lambda"),
    };
    println!("{}", out);
}
