use std::collections::HashMap;
use inquire::{Text, Select, InquireError, Confirm};
use inquire::{
    formatter::MultiOptionFormatter, MultiSelect,
};
use composer::ConfigSpec;
use snapshotter::Record;
use itertools::Itertools;

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

pub fn prompt_multi_names(records: Vec<Record>) -> HashMap<String, String> {
    let mut opts: Vec<String> = vec![];

    for rec in records {
        let opt = format!("{} - {}", &rec.namespace, &rec.version);
        opts.push(opt);
    }

    opts.sort();

    let options = opts.iter().map(String::as_str).collect();

    let formatter: MultiOptionFormatter<'_, &str> = &|a| format!("{} different namespaces", a.len());

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
        },
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

    let profile: Result<String, InquireError> =
        Select::new("Select Profile:", profiles)
        .without_help_message()
        .prompt();

    let sandbox = Text::new("Sandbox").with_default("stable").prompt();

    let sandbox = sandbox.unwrap();
    let profile = profile.unwrap();
    (profile, sandbox)
}
