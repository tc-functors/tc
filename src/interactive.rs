use composer::{
    ConfigSpec,
    Entity,
    Topology,
    spec::TestSpec,
};
use inquire::{
    Confirm,
    InquireError,
    MultiSelect,
    Select,
    Text,
    formatter::MultiOptionFormatter,
};
use itertools::Itertools;
use kit::*;
use snapshotter::Record;
use std::collections::HashMap;

pub fn prompt_versions(topologies: &HashMap<String, String>) -> (String, String, String, String) {
    let mut names: Vec<String> = topologies.keys().cloned().collect();

    names.sort();

    let topology: Result<String, InquireError> = Select::new("Topology name:", names)
        .with_page_size(20)
        .without_help_message()
        .prompt();

    let t = &topology.unwrap();
    let version = topologies.get(t).unwrap();

    let selected_version = Text::new("Version").with_default(version).prompt();

    let config = ConfigSpec::new(None);
    let roles = config.ci.roles;

    let mut profiles: Vec<String> = roles.keys().cloned().collect();
    profiles.sort();

    let profile: Result<String, InquireError> = Select::new("Select Profile:", profiles)
        .without_help_message()
        .prompt();

    let sandbox = Text::new("Sandbox").with_default("stable").prompt();

    let version = selected_version.unwrap();
    let sandbox = sandbox.unwrap();
    let profile = profile.unwrap();
    let msg = format!(
        "Do you want to deploy {}@{}.{}/{} ?",
        &t, &sandbox, &profile, &version
    );

    let ans = Confirm::new(&msg).with_default(false).prompt();

    match ans {
        Ok(true) => (t.to_string(), version, profile, sandbox),
        Ok(false) | Err(_) => {
            println!("Not deploying via CI. Exiting");
            std::process::exit(1);
        }
    }
}

pub fn prompt_names(topologies: &HashMap<String, String>) -> String {
    let mut names: Vec<String> = topologies.keys().cloned().collect();
    names.sort();

    let topology: Result<String, InquireError> = Select::new("Topology name:", names)
        .with_page_size(20)
        .without_help_message()
        .prompt();

    let t = &topology.unwrap();
    t.to_string()
}

pub fn prompt_multi_names(records: Vec<Record>) -> HashMap<String, String> {
    let mut opts: Vec<String> = vec![];

    for rec in records {
        let opt = format!("{} - {}", &rec.namespace, &rec.version);
        opts.push(opt);
    }

    opts.sort();

    let options = opts.iter().map(String::as_str).collect();

    let formatter: MultiOptionFormatter<'_, &str> =
        &|a| format!("{} different namespaces", a.len());

    let ans = MultiSelect::new("Select Topologies:", options)
        .with_formatter(formatter)
        .with_page_size(20)
        .without_help_message()
        .prompt();

    let mut res: HashMap<String, String> = HashMap::new();
    match ans {
        Ok(rs) => {
            for r in rs {
                let (ns, version) = r.split(" - ").collect_tuple().unwrap();
                res.insert(ns.to_string(), version.to_string());
            }
            res
        }
        Err(_) => {
            println!("Cannot process");
            std::process::exit(1);
        }
    }
}

pub fn prompt_env_sandbox() -> (String, String) {
    let config = ConfigSpec::new(None);
    let roles = config.ci.roles;

    let mut profiles: Vec<String> = roles.keys().cloned().collect();
    profiles.sort();

    let profile: Result<String, InquireError> = Select::new("Select Profile:", profiles)
        .without_help_message()
        .prompt();

    let sandbox = Text::new("Sandbox").with_default("stable").prompt();

    let sandbox = sandbox.unwrap();
    let profile = profile.unwrap();
    (profile, sandbox)
}

pub fn prompt_entity_components(topology: &Topology, entities: Vec<Entity>) -> Option<String> {
    let entities_str: Vec<String> = entities
        .into_iter()
        .map(|s| format!("{}s", s.to_str()))
        .collect();

    let entity: Result<String, InquireError> = Select::new("Select Entity:", entities_str)
        .with_page_size(10)
        .without_help_message()
        .prompt();

    let entity = &entity.unwrap();

    let components = match entity.as_ref() {
        "functions" => v![
            "Specific <function>",
            "vars",
            "roles",
            "layers",
            "concurrency",
            "runtime",
            "tags"
        ],
        "events" => v!["Specific <event>", "roles", "filters", "triggers"],
        "mutations" => v!["types", "roles"],
        "routes" => v!["Specific <route>", "gateway", "cors"],
        "states" => v!["definition", "tags", "logs"],
        "pages" => v!["code", "domains", "roles", "Edge functions"],
        _ => v!["nothing to do"],
    };

    let maybe_component: Result<String, InquireError> =
        Select::new("Select Component:", components)
            .with_page_size(10)
            .without_help_message()
            .prompt();

    let comp = &maybe_component.unwrap();

    let component = if comp.starts_with("Specific") {
        let names: Vec<String> = match entity.as_ref() {
            "functions" => topology.functions.keys().cloned().collect(),
            "events" => topology.events.keys().cloned().collect(),
            "routes" => topology.routes.keys().cloned().collect(),
            _ => vec!["nope".to_string()],
        };

        let name: Result<String, InquireError> = Select::new("Select Name:", names)
            .with_page_size(10)
            .without_help_message()
            .prompt();

        &name.unwrap()
    } else {
        comp
    };

    Some(format!("{}/{}", &entity, &component))
}

pub fn prompt_test_units(specs: HashMap<String, TestSpec>) -> (String, Option<TestSpec>) {
    let mut unit_names: Vec<String> = vec![];
    for (name, spec) in &specs {
        let m = format!("{} - {}", name, spec.entity.clone().unwrap());
        unit_names.push(m);
    }
    unit_names.sort();

    let name: Result<String, InquireError> = Select::new("Select Test Unit:", unit_names)
        .with_page_size(30)
        .without_help_message()
        .prompt();
    let name = name.unwrap();
    let (sname, _entity) = name.split(" - ").collect_tuple().unwrap();
    let p = specs.get(sname).cloned();
    (sname.to_string(), p)
}
