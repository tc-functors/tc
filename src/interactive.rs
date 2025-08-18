use std::collections::HashMap;
use inquire::{Text, Select, InquireError};
use composer::ConfigSpec;

pub fn prompt_versions(
    topologies: &HashMap<String, String>
) -> (String, String, String, String) {

    let mut names: Vec<String> =  topologies.keys().cloned().collect();

    names.sort();


    let topology: Result<String, InquireError> =
        Select::new("Topology name:", names)
        .with_page_size(20)
        .without_help_message()
        .prompt();

    let t = &topology.unwrap();
    let version = topologies.get(t).unwrap();

    let selected_version = Text::new("Version")
        .with_default(version)
        .prompt();

    let config = ConfigSpec::new(None);
    let roles = config.ci.roles;

    let mut profiles: Vec<String> = roles.keys().cloned().collect();
    profiles.sort();

    let profile: Result<String, InquireError> =
        Select::new("Select Profile:", profiles)
        .without_help_message()
        .prompt();

    let sandbox = Text::new("Sandbox").with_default("dev").prompt();

    (t.to_string(),
     selected_version.unwrap(),
     profile.unwrap(),
     sandbox.unwrap())
}

pub fn prompt_names(
    topologies: &HashMap<String, String>) -> String {

    let mut names: Vec<String> =  topologies.keys().cloned().collect();
    names.sort();

    let topology: Result<String, InquireError> =
        Select::new("Topology name:", names)
        .with_page_size(20)
        .without_help_message()
        .prompt();

    let t = &topology.unwrap();
    t.to_string()
}
