use composer::{
    Function,
};

use provider::{
    Auth,
    aws::agentcorectl,
    aws::agentcorectl::Runtime,

};
use std::collections::HashMap;


pub async fn create(auth: &Auth, function: &Function, _tags: &HashMap<String, String>) -> String {
    let client = agentcorectl::make_client(auth).await;
    let runtime = Runtime {
        name: function.name.clone(),
        langr: function.runtime.lang.to_str(),
        bucket: function.build.bucket.clone(),
        prefix: String::from("test"),
        role: function.runtime.role.name.clone(),
        handler: function.runtime.handler.clone()
    };
    println!("Creating function agentcore..");
    runtime.create_or_update(&client).await
}

pub async fn delete(_auth: &Auth, _function: &Function) {
    println!("Deleting agentcore..");
}
