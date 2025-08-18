use std::collections::HashMap;
use inquire::{Text, Select, InquireError, Confirm};
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

    let sandbox = Text::new("Sandbox").with_default("stable").prompt();

    let version = selected_version.unwrap();
    let sandbox = sandbox.unwrap();
    let profile = profile.unwrap();
    let msg = format!("Do you want to deploy {}@{}.{}/{} ?", &t, &sandbox, &profile, &version);

    let ans = Confirm::new(&msg)
        .with_default(false)
        .prompt();

    match ans {
        Ok(true) => {
            (t.to_string(),
             version,
             profile,
             sandbox)

        }
        Ok(false) | Err(_) => {
            println!("Not deploying via CI. Exiting");
            std::process::exit(1);
        }
    }
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
