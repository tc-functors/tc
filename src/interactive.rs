use compiler::Entity;
use compiler::spec::TestSpec;
use composer::{
    Topology,
};
use inquire::{
    InquireError,
    Select,
};
use itertools::Itertools;
use kit::*;
use std::collections::HashMap;

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
