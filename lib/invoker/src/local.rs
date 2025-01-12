use kit as u;
use kit::*;
use std::collections::HashMap;

pub async fn invoke(payload: &str) {
    let mut headers = HashMap::new();
    headers.insert(s!("content-type"), s!("application/json"));
    let url = "http://localhost:9000/2015-03-31/functions/function/invocations";
    let res = u::http_post(url, headers, payload.to_string()).await;
    let out = match res {
        Ok(r) => kit::pretty_json(&r),
        Err(_) => s!("Error invoking local lambda"),
    };
    println!("{}", out);
}
