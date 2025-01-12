use aws::eventbridge;
use serde_derive::{Deserialize, Serialize};
use tabled::{Style, Table, Tabled};

use aws::Env;
use kit as u;

#[derive(Tabled, Clone, Debug)]
struct Event {
    rule: String,
    event: String,
    target: String,
}

#[derive(Serialize, Deserialize)]
struct Pattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "detail-type")]
    detail_type: Option<Vec<String>>,
}

pub async fn list(env: &Env, service: &str) {
    let client = eventbridge::make_client(env).await;
    // fixme
    let bus = String::from("default");
    let rules = eventbridge::list_rules(client.clone(), bus.clone(), service.to_string()).await;

    let mut routes: Vec<Event> = vec![];
    for rule in rules {
        let p = rule.event_pattern.unwrap();
        let pattern: Pattern = serde_json::from_str(&p).unwrap();
        let rule_name = rule.name.unwrap();
        let target = eventbridge::get_target(client.clone(), bus.clone(), rule_name.clone()).await;
        let route = Event {
            rule: rule_name,
            event: u::maybe_vec_string(pattern.detail_type),
            target: u::split_last(&target, ":"),
        };
        routes.push(route)
    }

    let table = Table::new(routes).with(Style::psql()).to_string();
    println!("{}", table);
}
