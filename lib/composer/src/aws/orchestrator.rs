use super::event::Event;
use super::function::Function;
use super::mutation::Mutation;
use super::role::Role;
use super::function::Runtime;
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
pub struct Targets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<EventTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation: Option<MutationTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}

impl Default for Targets {
    fn default() -> Targets {
        Targets {
            event: None,
            mutation: None,
            function: None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Orchestrator {
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
    mutations: &HashMap<String, Mutation>
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
            _ => ()
        }
    }
    tx
}

fn make_orchestrator_code() -> String {
    format!("aW1wb3J0IGh0dHAuY2xpZW50CmltcG9ydCBqc29uCmltcG9ydCBib3RvMwoKZGVmIG1ha2VfbXV0YXRpb25fc3RyKG11dGF0aW9uX25hbWUsIGlucHV0LCBvdXRwdXQpOgogIGlucCA9ICcnCiAgZm9yIGssdiBpbiBpbnB1dC5pdGVtcygpOgogICAgaW5wICs9IGYnJHtrfTp7dn0sJwogIGlucCA9IGlucC5yc3RyaXAoJywnKQogIGZpZWxkcyA9ICcnCgogIG11dF9pbnB1dCA9ICcnCiAgZm9yIGssdiBpbiBpbnB1dC5pdGVtcygpOgogICAgbXV0X2lucHV0ICs9IGYne2t9OiR7a30sJwogIG11dF9pbnB1dCA9IG11dF9pbnB1dC5yc3RyaXAoJywnKQoKICBmb3IgayBpbiBvdXRwdXQua2V5cygpOgogICAgZmllbGRzICs9IGYne2t9ICcKICBmaWVsZHMgKz0gJ2NyZWF0ZWRBdCB1cGRhdGVkQXQnCiAgcXVlcnkgPSBmJ211dGF0aW9uKHtpbnB9KXt7e211dGF0aW9uX25hbWV9KHttdXRfaW5wdXR9KXt7e2ZpZWxkc319fX19JwogIHJldHVybiBxdWVyeQoKCmRlZiBtYWtlX211dGF0aW9uX3BheWxvYWQobXV0YXRpb25fbmFtZSwgaW5wdXQsIG91dHB1dCwgdmFyaWFibGVzKToKICBtdXRfc3RyID0gbWFrZV9tdXRhdGlvbl9zdHIobXV0YXRpb25fbmFtZSwgaW5wdXQsIG91dHB1dCkKICB2YXJpYWJsZXMgPSBqc29uLmR1bXBzKHZhcmlhYmxlcykKICBncmFwaHFsX211dGF0aW9uID0gewogICAgJ3F1ZXJ5JzogbXV0X3N0ciwKICAgICd2YXJpYWJsZXMnOiBmJ3t2YXJpYWJsZXN9JwogIH0KICByZXR1cm4ganNvbi5kdW1wcyhncmFwaHFsX211dGF0aW9uKQoKZGVmIHRyaWdnZXJfbXV0YXRpb24obXV0YXRpb25fbWV0YWRhdGEsIHZhcmlhYmxlcyk6CiAgbXV0YXRpb25fbmFtZSA9IG11dGF0aW9uX21ldGFkYXRhLmdldCgnbmFtZScpCiAgaW5wdXQgPSBtdXRhdGlvbl9tZXRhZGF0YS5nZXQoJ2lucHV0JykKICBvdXRwdXQgPSBtdXRhdGlvbl9tZXRhZGF0YS5nZXQoJ291dHB1dCcpCiAgZW5kcG9pbnQgPSBtdXRhdGlvbl9tZXRhZGF0YS5nZXQoJ2VuZHBvaW50JykKICBhcGlfa2V5ID0gbXV0YXRpb25fbWV0YWRhdGEuZ2V0KCdhcGlfa2V5JykKICBob3N0ID0gZW5kcG9pbnQucmVwbGFjZSgnaHR0cHM6Ly8nLCcnKS5yZXBsYWNlKCcvZ3JhcGhxbCcsJycpCgogIGNvbm4gPSBodHRwLmNsaWVudC5IVFRQU0Nvbm5lY3Rpb24oaG9zdCwgNDQzKQogIGhlYWRlcnMgPSB7CiAgICAgICAgJ0NvbnRlbnQtdHlwZSc6ICdhcHBsaWNhdGlvbi9ncmFwaHFsJywKICAgICAgICAneC1hcGkta2V5JzogYXBpX2tleSwKICAgICAgICAnaG9zdCc6IGhvc3QKICAgIH0KICByZXN0X3BheWxvYWQgPSBtYWtlX211dGF0aW9uX3BheWxvYWQobXV0YXRpb25fbmFtZSwgaW5wdXQsIG91dHB1dCwgdmFyaWFibGVzKQogIHByaW50KHJlc3RfcGF5bG9hZCkKICBjb25uLnJlcXVlc3QoJ1BPU1QnLCAnL2dyYXBocWwnLCByZXN0X3BheWxvYWQsIGhlYWRlcnMpCiAgcmVzcG9uc2UgPSBjb25uLmdldHJlc3BvbnNlKCkKICByZXNwb25zZV9zdHJpbmcgPSByZXNwb25zZS5yZWFkKCkuZGVjb2RlKCd1dGYtOCcpCiAgcHJpbnQocmVzcG9uc2Vfc3RyaW5nKQogIHJldHVybiByZXNwb25zZV9zdHJpbmcKCmRlZiB0cmlnZ2VyX2V2ZW50KGV2ZW50X21ldGFkYXRhLCBwYXlsb2FkKToKICBjbGllbnQgPSBib3RvMy5jbGllbnQoJ2V2ZW50cycpCiAgcmVzID0gY2xpZW50LnB1dF9ldmVudHMoCiAgICBFbnRyaWVzPVsKICAgICAgewogICAgICAgICdTb3VyY2UnOiBldmVudF9tZXRhZGF0YS5nZXQoJ3NvdXJjZScpLAogICAgICAgICdFdmVudEJ1c05hbWUnOiBldmVudF9tZXRhZGF0YS5nZXQoJ2J1cycpLAogICAgICAgICdEZXRhaWwnOiBqc29uLmR1bXBzKHBheWxvYWQpLAogICAgICAgICdEZXRhaWxUeXBlJzogZXZlbnRfbWV0YWRhdGEuZ2V0KCduYW1lJykKICAgICAgfQogICAgXQogICkKICBwcmludChyZXMpCiAgcmV0dXJuIHJlcwoKZGVmIHRyaWdnZXJfZnVuY3Rpb24oZnVuY3Rpb25fYXJuLCBwYXlsb2FkKToKICBjbGllbnQgPSBib3RvMy5jbGllbnQoJ2xhbWJkYScpCiAgcmVzcG9uc2UgPSBjbGllbnQuaW52b2tlX2FzeW5jKAogICAgICAgIEZ1bmN0aW9uTmFtZT1mdW5jdGlvbl9hcm4sCiAgICAgICAgSW52b2tlQXJncz1qc29uLmR1bXBzKHBheWxvYWQpCiAgKQogIHByaW50KHJlc3BvbnNlKQogIHJldHVybiByZXNwb25zZQoKZGVmIHRyaWdnZXJfdGFyZ2V0cyh0YXJnZXRzLCBwYXlsb2FkKToKICBldmVudF9tZXRhZGF0YSA9IHRhcmdldHMuZ2V0KCJldmVudCIpCiAgbXV0YXRpb25fbWV0YWRhdGEgPSB0YXJnZXRzLmdldCgibXV0YXRpb24iKQogIGZ1bmN0aW9uX2FybiA9IHRhcmdldHMuZ2V0KCJmdW5jdGlvbiIpCiAgaWYgZXZlbnRfbWV0YWRhdGEgaXMgbm90IE5vbmU6CiAgICB0cmlnZ2VyX2V2ZW50KGV2ZW50X21ldGFkYXRhLCBwYXlsb2FkKQoKICBpZiBmdW5jdGlvbl9hcm4gaXMgbm90IE5vbmU6CiAgICB0cmlnZ2VyX2Z1bmN0aW9uKGZ1bmN0aW9uX2FybiwgcGF5bG9hZCkKCiAgaWYgbXV0YXRpb25fbWV0YWRhdGEgaXMgbm90IE5vbmU6CiAgICB0cmlnZ2VyX211dGF0aW9uKG11dGF0aW9uX21ldGFkYXRhLCBwYXlsb2FkKQoKICByZXR1cm4gVHJ1ZQoKCmRlZiBsb2FkX21ldGFkYXRhKHNvdXJjZV9hcm4pOgogIHdpdGggb3Blbignb3JjaGVzdHJhdG9yLmpzb24nKSBhcyBqc29uX2RhdGE6CiAgICBkID0ganNvbi5sb2FkKGpzb25fZGF0YSkKICAgIHRhcmdldHMgPSBkLmdldCgndGFyZ2V0cycpLmdldChzb3VyY2VfYXJuKQogICAgcHJpbnQodGFyZ2V0cykKICAgIGpzb25fZGF0YS5jbG9zZSgpCiAgICByZXR1cm4gdGFyZ2V0cwoKZGVmIG1ha2VfaW5wdXQocmVzcG9uc2VfcGF5bG9hZCk6CiAgaWYgJ2RldGFpbCcgaW4gcmVzcG9uc2VfcGF5bG9hZCBhbmQgJ2RldGFpbC10eXBlJyBpbiByZXNwb25zZV9wYXlsb2FkOgogICAgcmV0dXJuIHJlc3BvbnNlX3BheWxvYWQuZ2V0KCdkZXRhaWwnKQogIGVsc2U6CiAgICByZXR1cm4gcmVzcG9uc2VfcGF5bG9hZAoKCmRlZiBoYW5kbGVyKGV2ZW50LCBjb250ZXh0KToKICBwcmludChldmVudCkKICBpbnB1dCA9IG1ha2VfaW5wdXQoZXZlbnQuZ2V0KCdyZXNwb25zZVBheWxvYWQnKSkKICBzID0gZXZlbnQuZ2V0KCdyZXF1ZXN0Q29udGV4dCcpLmdldCgnZnVuY3Rpb25Bcm4nKQogIHNvdXJjZV9hcm4gPSBzLnJzcGxpdCgnOicsIDEpWzBdCiAgdGFyZ2V0cyA9IGxvYWRfbWV0YWRhdGEoc291cmNlX2FybikKICByZXMgPSB0cmlnZ2VyX3RhcmdldHModGFyZ2V0cywgaW5wdXQpCiAgcmV0dXJuIHJlcwo=")
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

impl Orchestrator {
    pub fn new(
        namespace: &str,
        fns: &HashMap<String, Function>,
        events: &HashMap<String, Event>,
        mutations: &HashMap<String, Mutation>

    ) -> Option<Orchestrator> {

        let mut txs: HashMap<String, Targets> = HashMap::new();

        let mut target_count = 0;
        for (_, f) in fns {
            target_count += f.targets.len();
        }

        if target_count == 0 {
            return None
        }

        for (_, f) in fns {
            let targets = make_targets(namespace, f, events, mutations);
            let arn = template::lambda_arn(&f.fqn);
            txs.insert(arn, targets);
        }

        let orch_name = format!("{}_tc-orchestrator_{{{{sandbox}}}}", namespace);
        let arn = template::lambda_arn(&orch_name);
        let function = make_function(namespace, &orch_name, &orch_name);

        let orch = Orchestrator {
            namespace: namespace.to_string(),
            name: orch_name.clone(),
            arn: arn,
            function: function,
            targets: txs
        };
        Some(orch)
    }

    pub fn dump(&self, config: &HashMap<String, String>) {
        let dir = format!("/tmp/tc/{}", self.namespace);
        let json = serde_json::to_string(&self).unwrap();

        let mut table: HashMap<&str, &str> = HashMap::new();
        for (k, v) in config {
            table.insert(&k, &v);
        }
        let rs = u::stencil(&json, table);

        let orch_path = format!("{}/orchestrator.json", &dir);
        let code_path = format!("{}/handler.py", &dir);

        let b64_code = make_orchestrator_code();
        let bytes = general_purpose::STANDARD.decode(&b64_code).unwrap();
        let code = String::from_utf8_lossy(&bytes);

        u::sh(&format!("mkdir -p {}", &dir), &u::pwd());
        u::write_str(&orch_path, &rs);
        u::write_str(&code_path, &code);
    }

    pub fn clean(&self) {
        let dir = format!("/tmp/tc/{}", self.namespace);
        u::sh(&format!("rm -rf {}", &dir), &u::pwd());
    }

}
