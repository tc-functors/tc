use super::event::Event;
use super::function::Function;
use super::mutation::Mutation;
use super::channel::Channel;
use super::role::Role;
use super::function::Runtime;
use super::function::code;
use super::function::Build;
use compiler::Entity;
use compiler::{LangRuntime, BuildKind};
use compiler::spec::function::Provider;
use super::template;
use crate::tag;

use base64::{
    Engine as _,
    engine::general_purpose,
};

use std::collections::HashMap;

use serde_derive::{
    Deserialize,
    Serialize,
};
use kit::*;
use kit as u;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MutationTarget {
    pub name: String,
    pub input: Option<HashMap<String, String>>,
    pub output: Option<HashMap<String, String>>,
    pub endpoint: String,
    pub api_key: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventTarget {
    pub name: String,
    pub source: String,
    pub bus: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelTarget {
    pub name: String,
    pub http_domain: String,
    pub api_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Targets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<EventTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation: Option<MutationTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
   #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<ChannelTarget>,
}

impl Default for Targets {
    fn default() -> Targets {
        Targets {
            event: None,
            mutation: None,
            function: None,
            channel: None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transducer {
    pub namespace: String,
    pub name: String,
    pub arn: String,
    pub function: Function,
    pub targets: HashMap<String, Targets>
}


fn make_targets(
    namespace: &str,
    f: &Function,
    events: &HashMap<String, Event>,
    mutations: &HashMap<String, Mutation>,
    channels: &HashMap<String, Channel>
) -> Targets {

    let mut tx: Targets = Targets::default();

    for target in &f.targets {

        match target.entity {
            Entity::Event => {
                if let Some(event) = events.get(&target.name) {
                    let source = match event.pattern.source.first() {
                        Some(s) => s,
                        None => "default"
                    };
                    let t = EventTarget {
                        name: target.name.clone(),
                        source: source.to_string(),
                        bus: event.bus.clone()
                    };
                    tx.event = Some(t);
                } else {
                    tx.event = None;
                }

            },
            Entity::Mutation => {
                let maybe_mut = mutations.get("default");
                match maybe_mut {
                    Some(m) => {
                        if let Some(resolver) = m.resolvers.get(&target.name) {
                            let types_map = &m.types_map;
                            let input = &resolver.input;
                            let output = &resolver.output;
                            let input_schema = types_map.get(input);
                            let output_schema = types_map.get(output);
                            let t = MutationTarget {
                                input: input_schema.cloned(),
                                output: output_schema.cloned(),
                                name: target.name.clone(),
                                endpoint: format!("{{{{GRAPHQL_ENDPOINT}}}}"),
                                api_key: format!("{{{{GRAPHQL_API_KEY}}}}"),
                            };
                            tx.mutation = Some(t);
                        }

                    },
                    None => {
                        tx.mutation = None;
                    }
                }
            },
            Entity::Function => {
                let fqn = template::lambda_fqn(namespace, &target.name);
                tx.function = Some(template::lambda_arn(&fqn))
            },
            Entity::Channel => {
                if let Some(channel) = channels.get(&target.name) {
                    let t = ChannelTarget {
                        name: channel.name.clone(),
                        http_domain: format!("{{{{HTTP_DOMAIN}}}}"),
                        api_key: format!("{{{{API_KEY}}}}")
                    };
                    tx.channel = Some(t);
                }
            }
            _ => ()
        }
    }
    tx
}

fn make_function(namespace: &str, name: &str, fqn: &str) -> Function {

    let dir = format!("/tmp/tc/{}", namespace);

    let uri = format!("{}/lambda.zip", &dir);
    let role = Role::default(Entity::Function);

    let build = Build {
        dir: dir.to_string(),
        kind: BuildKind::Code,
        pre: vec![],
        post: vec![],
        version: None,
        command: s!("zip -9 -q lambda.zip *.py *.json"),
        shared_context: false,
        skip_dev_deps: false,
        environment: HashMap::new()
    };

    let tags = tag::make(namespace, "");

    let runtime = Runtime {
        lang: LangRuntime::Python311,
        provider: Provider::Lambda,
        handler: s!("handler.handler"),
        package_type: s!("zip"),
        uri: uri,
        layers: vec![],
        environment: HashMap::new(),
        tags: tags,
        provisioned_concurrency: None,
        reserved_concurrency: None,
        role: role,
        memory_size: Some(128),
        cpu: None,
        timeout: Some(60),
        snapstart: false,
        enable_fs: false,
        network: None,
        fs: None,
        infra_spec: HashMap::new(),
        cluster: String::from("")
    };

    Function {
        name: name.to_string(),
        actual_name: name.to_string(),
        arn: template::lambda_arn(&fqn),
        version: s!(""),
        fqn: fqn.to_string(),
        description: None,
        dir: dir.to_string(),
        namespace: namespace.to_string(),
        runtime: runtime,
        build: build,
        layer_name: None,
        targets: vec![],
        test: HashMap::new()
    }
}

impl Transducer {
    pub fn new(
        namespace: &str,
        fns: &HashMap<String, Function>,
        events: &HashMap<String, Event>,
        mutations: &HashMap<String, Mutation>,
        channels: &HashMap<String, Channel>

    ) -> Option<Transducer> {

        let mut txs: HashMap<String, Targets> = HashMap::new();

        let mut target_count = 0;
        for (_, f) in fns {
            target_count += f.targets.len();
        }

        if target_count == 0 {
            return None
        }

        for (_, f) in fns {
            let targets = make_targets(namespace, f, events, mutations, channels);
            let arn = template::lambda_arn(&f.fqn);
            txs.insert(arn, targets);
        }

        let tname = format!("{}_transducer_{{{{sandbox}}}}", namespace);
        let arn = template::lambda_arn(&tname);
        let function = make_function(namespace, &tname, &tname);

        let transducer = Transducer {
            namespace: namespace.to_string(),
            name: tname.clone(),
            arn: arn,
            function: function,
            targets: txs
        };
        Some(transducer)
    }

    pub fn dump(&self, config: &HashMap<String, String>) {
        let dir = format!("/tmp/tc/{}", self.namespace);
        let json = serde_json::to_string(&self).unwrap();

        let mut table: HashMap<&str, &str> = HashMap::new();
        for (k, v) in config {
            table.insert(&k, &v);
        }
        let rs = u::stencil(&json, table);

        let t_path = format!("{}/transducer.json", &dir);
        let code_path = format!("{}/handler.py", &dir);

        let b64_code = code::make_transducer_code();
        let bytes = general_purpose::STANDARD.decode(&b64_code).unwrap();
        let code = String::from_utf8_lossy(&bytes);

        u::sh(&format!("mkdir -p {}", &dir), &u::pwd());
        u::write_str(&t_path, &rs);
        u::write_str(&code_path, &code);
    }

    pub fn clean(&self) {
        let dir = format!("/tmp/tc/{}", self.namespace);
        u::sh(&format!("rm -rf {}", &dir), &u::pwd());
    }

}
