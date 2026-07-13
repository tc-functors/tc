use crate::aws::{
    lambda,
    microvm,
};
use compiler::function::Provider;
use composer::Function;
use kit as u;
use kit::*;
use provider::Auth;
use std::collections::HashMap;

async fn call_endpoint(endpoint: &str, token: &str, _payload: &str) {
    let url = format!("https://{}", endpoint);
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert(s!("x-aws-proxy-auth"), token.to_string());
    headers.insert(s!("content-type"), s!("application/json"));
    headers.insert(s!("accept"), s!("application/json"));
    headers.insert(
        s!("user-agent"),
        s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
    );
    let res = u::http_get(&url, headers).await;
    println!("{}", u::pretty_json(&res));
}

pub async fn invoke(auth: &Auth, f: &Function, payload: &str) {
    match f.runtime.provider {
        Provider::Lambda => {
            let name = &f.fqn;
            let client = lambda::make_client(auth).await;
            println!("Invoking function {}", &name);
            let _ = lambda::invoke(client, &name, payload).await;
        }
        Provider::MicroVm => {
            let client = microvm::make_client(auth).await;
            let image_name = f.build.image_name.clone();
            let maybe_microvm = microvm::find(&client, &image_name).await;
            if let Some(microvm_id) = maybe_microvm {
                tracing::debug!("Invoking microvm {}", microvm_id);
                if let Some(token) = microvm::get_token(&client, &microvm_id, 30).await {
                    let run_info = microvm::get_microvm(&client, &microvm_id).await;
                    call_endpoint(&run_info.endpoint, &token, payload).await;
                }
            } else {
                println!("No microvm running")
            }
        }

        Provider::AgentCore => todo!(),
    }
}

pub async fn invoke_emulator(payload: &str) {
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
