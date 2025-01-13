use aws::{eventbridge, sfn};
use aws::Env;
use kit as u;
use kit::*;
use colored::Colorize;
use std::io::stdout;

fn target_id(name: &str) -> String {
    format!("{}_target", name)
}

pub fn role_arn(acc: &str, namespace: &str, sandbox: &str, id: &str) -> String {
    format!(
        "arn:aws:iam::{}:role/{}-{}-{}-role",
        acc, namespace, sandbox, id
    )
}

pub async fn route(env: &Env, event_id: &str, service: &str, sandbox: &str, rule: &str) {
    let client = eventbridge::make_client(env).await;
    let bus = &env.config.eventbridge.bus;
    let target_name = format!("{}_{}", service, sandbox);
    let target_id = target_id(event_id);
    let target_arn = env.sfn_arn(&target_name);
    let role = role_arn(&env.account(), service, sandbox, "event");
    let target = eventbridge::make_target(&target_id, event_id, &target_arn, &role, None, None);
    println!("Routing {} to {}", event_id, target_name);
    eventbridge::put_target(client, bus.to_string(), rule.to_string(), target).await;
}

pub async fn freeze(env: &Env, name: &str) {
    let mut log_update = LogUpdate::new(stdout()).unwrap();
    let client = sfn::make_client(env).await;
    let arn = env.sfn_arn(name);
    let version = sfn::get_tag(&client, &arn, s!("version")).await;
    if &version != "0.0.1" && !&version.is_empty() {
        let _ = log_update.render(&format!("Freezing {} ({})", name, version.blue() ));
        let kv = u::kv("freeze", "true");
        let _ = sfn::update_tags(&client, &arn, kv).await;
    }
}

pub async fn unfreeze(env: &Env, name: &str) {
    let mut log_update = LogUpdate::new(stdout()).unwrap();
    let client = sfn::make_client(env).await;
    let arn = env.sfn_arn(name);
    let version = sfn::get_tag(&client, &arn, s!("version")).await;
    if &version != "0.0.1" && !&version.is_empty() {
        let _ = log_update.render(&format!("Unfreezing {} ({})", name, version.blue() ));
        let kv = u::kv("freeze", "false");
        let _ = sfn::update_tags(&client, &arn, kv).await;
    }
}

pub fn should_abort(sandbox: &str) -> bool {
    let yes = match std::env::var("CIRCLECI") {
        Ok(_) => false,
        Err(_) => match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => true
        }
    };
    yes && ( sandbox == "stable")
}

pub fn guard(sandbox: &str) {
    if should_abort(sandbox) {
        std::panic::set_hook(Box::new(|_| {
            println!("Cannot create stable sandbox outside CI");
        }));
        panic!("Cannot create stable sandbox outside CI")
    }
}
