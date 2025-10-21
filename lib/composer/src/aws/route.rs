use crate::aws::{
    event::Event,
    function::Function,
    queue::Queue,
    role::Role,
    template,
};
use compiler::{
    Entity,
    spec::{
        RouteSpec,
        TopologySpec,
        route::CorsSpec,
    },
};
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cors {
    pub methods: Vec<String>,
    pub origins: Vec<String>,
    #[serde(alias = "headers", alias = "allowed_headers")]
    pub headers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Authorizer {
    pub create: bool,
    pub name: String,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub skip: bool,
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: Option<Authorizer>,
    pub role_arn: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
    pub is_async: bool,
    pub cors: Cors,
    pub target: Target,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub name: String,
    pub arn: String,
    pub request_params: HashMap<String, String>,
}

fn _make_response_template() -> String {
    format!(r#"#set ($parsedPayload = $util.parseJson($input.json('$.output'))) $parsedPayload"#)
}

fn make_request_template(method: &str, request_template: Option<String>) -> String {
    if method == "POST" {
        match request_template {
            Some(r) => match r.as_ref() {
                "detail" => s!(
                    "\"{\"path\": \"${request.path}\", \"detail\": ${request.body.detail}, \"method\": \"${context.httpMethod}\"}\""
                ),
                "merged" => s!("\"{\"path\": $request.path, \"body\": $request.body}\""),
                _ => r,
            },
            None => s!("${request.body}"),
        }
    } else {
        s!("\"{\"path\": \"${request.path}\", \"method\": \"${context.httpMethod}\"}\"")
    }
}

fn find_function(f: &str, fns: &HashMap<String, Function>) -> String {
    match fns.get(f) {
        Some(_) => template::maybe_namespace(f),
        None => f.to_string(),
    }
}

fn make_target(
    fqn: &str,
    rspec: &RouteSpec,
    method: &str,
    fns: &HashMap<String, Function>,
    events: &HashMap<String, Event>,
    queues: &HashMap<String, Queue>,
) -> Target {
    if let Some(f) = &rspec.function {
        let name = find_function(&f, fns);
        return Target {
            entity: Entity::Function,
            name: name.clone(),
            arn: template::lambda_arn(&name),
            request_params: HashMap::new(),
        };
    } else if let Some(ev) = &rspec.event {
        let mut req: HashMap<String, String> = HashMap::new();
        if let Some(event) = events.get(ev) {
            let pattern = event.pattern.clone();
            let detail = match method {
                "GET" => s!("${request.path}"),
                "POST" => s!("${request.body}"),
                _ => s!("${request.path}"),
            };
            let source = match pattern.source.first() {
                Some(s) => s.clone(),
                None => s!("default"),
            };
            let detail_type = pattern.detail_type.first().unwrap();
            req.insert(s!("Detail"), detail);
            req.insert(s!("DetailType"), detail_type.clone());
            req.insert(s!("Source"), source);
            req.insert(s!("EventBusName"), event.bus.clone());
        } else {
            panic!("No event defined {}", &ev)
        }
        return Target {
            entity: Entity::Event,
            name: s!(ev),
            arn: String::from(""),
            request_params: req,
        };
    } else if let Some(q) = &rspec.queue {
        let mut req: HashMap<String, String> = HashMap::new();
        if let Some(queue) = queues.get(q) {
            req.insert(s!("QueueUrl"), template::sqs_url(&queue.name));
            req.insert(s!("MessageBody"), format!("{{{{payload}}}}"));
        } else {
            panic!("No queue defined {}", &q)
        }
        return Target {
            entity: Entity::Queue,
            name: s!(q),
            arn: String::from(""),
            request_params: req,
        };
    } else {
        let arn = template::sfn_arn(fqn);
        let input = make_request_template(method, rspec.request_template.clone());
        let mut req: HashMap<String, String> = HashMap::new();
        req.insert(s!("StateMachineArn"), s!(arn));
        req.insert(s!("Name"), fqn.to_string());
        req.insert(s!("Input"), input);
        return Target {
            entity: Entity::State,
            name: fqn.to_string(),
            arn: arn,
            request_params: req,
        };
    }
}

fn make_cors(maybe_cors: &Option<CorsSpec>) -> Cors {
    match maybe_cors {
        Some(c) => Cors {
            methods: {
                if c.methods.is_empty() {
                    v!["*"]
                } else {
                    c.methods.clone()
                }
            },
            origins: {
                if c.origins.is_empty() {
                    v!["*"]
                } else {
                    c.origins.clone()
                }
            },
            headers: c.headers.clone().unwrap_or(v!["*"])
        },
        None => Cors {
            methods: v!["*"],
            origins: v!["*"],
            headers: v!["*"]
        }
    }
}

fn make_authorizer(
    fqn: &str,
    rspec: &RouteSpec,
    fns: &HashMap<String, Function>,
) -> Option<Authorizer> {
    if let Some(azer) = &rspec.authorizer {
        match fns.get(azer) {
            Some(_) => {
                if azer.contains("{{") {
                    Some(Authorizer {
                        create: false,
                        name: azer.to_string(),
                        kind: s!("lambda"),
                    })
                } else {
                    Some(Authorizer {
                        create: true,
                        name: template::maybe_namespace(&azer),
                        kind: s!("lambda"),
                    })
                }
            },
            None => {
                if azer == "cognito" {
                    Some(Authorizer {
                        create: true,
                        name: fqn.to_string(),
                        kind: s!("cognito")
                    })
                } else {
                    Some(Authorizer {
                        create: false,
                        name: azer.to_string(),
                        kind: s!("lambda")
                    })
                }
            }
        }
    } else {
        None
    }
}


impl Route {
    pub fn new(
        fqn: &str,
        name: &str,
        _spec: &TopologySpec,
        rspec: &RouteSpec,
        fns: &HashMap<String, Function>,
        events: &HashMap<String, Event>,
        queues: &HashMap<String, Queue>,
        skip: bool,
    ) -> Route {
        let gateway = match &rspec.gateway {
            Some(gw) => gw.clone(),
            None => s!(fqn),
        };

        let path = match &rspec.path {
            Some(p) => p.clone(),
            None => s!(name),
        };

        let method = match &rspec.method {
            Some(m) => m.clone(),
            None => s!("POST"),
        };

        let is_async = match rspec.is_async {
            Some(s) => s,
            None => false,
        };

        let stage = match &rspec.stage {
            Some(s) => s.clone(),
            None => s!("$default"),
        };

        let target = make_target(fqn, rspec, &method, fns, events, queues);

        let authorizer = make_authorizer(fqn, rspec, fns);

        let cors = make_cors(&rspec.cors);

        Route {
            method: method.clone(),
            path: path,
            gateway: gateway,
            authorizer: authorizer,
            target: target,
            role_arn: Role::entity_role_arn(Entity::Route),
            stage: stage,
            stage_variables: HashMap::new(),
            is_async: is_async,
            cors: cors,
            skip: skip,
        }
    }
}
