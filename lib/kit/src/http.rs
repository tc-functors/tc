use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Error;
use serde_json::Value;
use std::collections::HashMap;

use crate::io;
use crate::json::{json_value, json_value_safe};

fn as_headers(h: HashMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    for (k, v) in h.into_iter() {
        let k = k.to_owned();
        let key = HeaderName::from_lowercase(k.as_bytes()).unwrap();
        let val = HeaderValue::from_str(&v).unwrap();
        headers.insert(key, val);
    }

    headers
}

pub async fn http_post(
    url: &str,
    headers: HashMap<String, String>,
    body: String,
) -> Result<Value, Error> {
    let client = reqwest::Client::new();
    let headers = as_headers(headers);
    let response = client
        .post(url)
        .headers(headers)
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    match response {
        Ok(res) => {
            if res == "ok" {
                Ok(serde_json::json!(res))
            } else {
                Ok(json_value_safe(&res))
            }
        }
        Err(_) => Ok(serde_json::json!("error")),
    }
}

pub async fn http_get(url: &str, headers: HashMap<String, String>) -> Value {
    let client = reqwest::Client::new();
    let headers = as_headers(headers);
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    json_value(&response)
}

pub async fn download(url: &str, headers: HashMap<String, String>, outfile: &str) {
    let client = reqwest::Client::new();
    let headers = as_headers(headers);
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let bytea = response.to_vec();
    io::write_bytes(outfile, bytea);
}

pub async fn upload(url: &str, headers: HashMap<String, String>, path: &str) {
    let bytea = io::read_bytes(path);
    let headers = as_headers(headers);
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .headers(headers)
        .body(bytea)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{:?}", response);
}
