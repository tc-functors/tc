use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub enum Entity {
    #[serde(alias = "function")]
    Function,
    #[serde(alias = "queue")]
    Queue,
    #[serde(alias = "route")]
    Route,
    #[serde(alias = "channel")]
    Channel,
    #[serde(alias = "event")]
    Event,
    #[serde(alias = "state")]
    State,
    #[serde(alias = "mutation")]
    Mutation,
    #[serde(alias = "trigger")]
    Trigger,
    #[serde(alias = "schedule")]
    Schedule,
    #[serde(alias = "page")]
    Page,
}

impl FromStr for Entity {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "function" | "functions" => Ok(Entity::Function),
            "queue" | "queues" => Ok(Entity::Queue),
            "route" | "routes" => Ok(Entity::Route),
            "channel" | "channels" => Ok(Entity::Channel),
            "event" | "events" => Ok(Entity::Event),
            "state" | "states" | "flow" => Ok(Entity::State),
            "mutation" | "mutations" => Ok(Entity::Mutation),
            "trigger" | "triggers" => Ok(Entity::Trigger),
            "schedule" | "schedules" => Ok(Entity::Schedule),
            "page" | "pages" => Ok(Entity::Page),
            _ => Ok(Entity::Function),
        }
    }
}

impl Entity {
    pub fn to_str(&self) -> String {
        match self {
            Entity::Function => s!("function"),
            Entity::Queue => s!("queue"),
            Entity::Route => s!("route"),
            Entity::Channel => s!("channel"),
            Entity::Event => s!("event"),
            Entity::Mutation => s!("mutation"),
            Entity::State => s!("state"),
            Entity::Trigger => s!("trigger"),
            Entity::Schedule => s!("schedule"),
            Entity::Page => s!("page"),
        }
    }

    pub fn as_entity_component(s: &str) -> (Entity, Option<String>) {
        match s.split_once("/") {
            Some((k, v)) => (Entity::from_str(&k).unwrap(), Some(v.to_string())),
            _ => (Entity::from_str(&s).unwrap(), None),
        }
    }

    pub fn print() {
        let v: Vec<&str> = vec![
            "events",
            "functions",
            "roles",
            "routes",
            "flow",
            "mutations",
            "schedules",
            "queues",
            "channels",
            "pools",
            "pages",
        ];
        for x in v {
            println!("{x}");
        }
    }

    pub fn as_entity(s: Option<String>) -> Option<Entity> {
        match s {
            Some(p) => Some(Entity::from_str(&p).unwrap()),
            None => None,
        }
    }

    pub fn from_arn(arn: &str) -> Option<Entity> {
        let parts: Vec<&str> = arn.split(":").collect();
        let s = parts.into_iter().nth(2).unwrap_or_default();
        match s {
            "lambda" => Some(Entity::Function),
            "states" => Some(Entity::State),
            "appsync" => Some(Entity::Mutation),
            _ => None,
        }
    }
}
