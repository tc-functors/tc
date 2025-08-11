use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Principal {
    #[serde(rename(serialize = "Service", deserialize = "Service"))]
    service: Vec<String>
}

impl Principal {

    fn new() -> Principal {
        Principal {
            service: vec![
                s!("lambda.amazonaws.com"),
                s!("events.amazonaws.com"),
                s!("states.amazonaws.com"),
                s!("logs.amazonaws.com"),
                s!("apigateway.amazonaws.com"),
                s!("appsync.amazonaws.com"),
                s!("scheduler.amazonaws.com")
            ]
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Statement {
    #[serde(rename(serialize = "Effect", deserialize = "Effect"))]
    effect: String,
    #[serde(rename(serialize = "Principal", deserialize = "Principal"))]
    principal: Principal,
    #[serde(rename(serialize = "Action", deserialize = "Action"))]
    action: String,

}
impl Statement {

    fn new() -> Statement {
        let principal = Principal::new();
        Statement {
            effect: s!("Allow"),
            principal: principal,
            action: s!("sts:AssumeRole")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Trust {
    #[serde(rename(serialize = "Version", deserialize = "Version"))]
    pub version: String,
    #[serde(rename(serialize = "Statement", deserialize = "Statement"))]
    pub statement: Vec<Statement>
}

impl Trust {

    pub fn new() -> Trust {
        let statement = Statement::new();
        Trust {
            version: s!("2012-10-17"),
            statement: vec![statement]
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
