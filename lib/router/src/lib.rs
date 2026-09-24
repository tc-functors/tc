use configurator::Config;
use provider::{
    Auth,
    aws::eventbridge,
    aws::gateway
};
use deployer::aws;
use deployer::aws::route::Gateway;
use composer::Topology;

fn target_id(name: &str) -> String {
    format!("{}_target", name)
}

pub fn role_arn(acc: &str, namespace: &str, sandbox: &str, id: &str) -> String {
    format!(
        "arn:aws:iam::{}:role/{}-{}-{}-role",
        acc, namespace, sandbox, id
    )
}

pub async fn route_event(auth: &Auth, event_id: &str, service: &str, sandbox: &str, rule: &str) {
    let client = eventbridge::make_client(auth).await;
    let config = Config::new();
    let bus = &config.aws.eventbridge.bus;
    let target_name = format!("{}_{}", service, sandbox);
    let target_id = target_id(event_id);
    let target_arn = auth.sfn_arn(&target_name);
    let role = role_arn(&auth.account, service, sandbox, "event");
    let target = eventbridge::make_target(
        &target_id,
        event_id,
        &target_arn,
        &role,
        None,
        None,
        None,
        None,
        None,
    );
    println!("Routing {} to {}", event_id, target_name);
    eventbridge::put_target(client, bus.to_string(), rule.to_string(), target).await;
}

pub async fn route(auth: &Auth, topology: &Topology, sandbox: &str) {
    let routes = topology.routes.clone();
    let gateways = aws::route::collate_gateways(&routes, &auth.name, sandbox);
    let client = gateway::make_client(auth).await;
    for (name, gateway) in gateways {
        let Gateway {
            stage,
            domain,
            paths,
            manage,
            ..
        } = gateway;
        if manage {
            let maybe_api_id = gateway::find_api(&client, &name).await;
            if let Some(api_id) = maybe_api_id {
                if let Some(dom) = domain {
                    aws::route::update_dns(auth, sandbox, &api_id, &stage, &dom, paths).await
                }
            }
        }
    }
}
