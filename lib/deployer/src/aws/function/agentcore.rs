use composer::{
    Function,
};

use provider::{
    Auth,
};
use std::collections::HashMap;


pub async fn _create(_auth: &Auth, _function: &Function, _tags: &HashMap<String, String>) {
    println!("Creating agentcore..");
}

pub async fn _delete(_auth: &Auth, _function: &Function) {
    println!("Deleting agentcore..");
}
