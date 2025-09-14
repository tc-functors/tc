use configurator::Config;
use inquire::{
    Confirm,
    InquireError,
    Select,
    Text,
};
use kit as u;
use std::collections::HashMap;

// interactive

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

    let config = Config::new(None);
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

pub async fn deploy_interactive() {
    let dir = u::root();
    u::sh("git fetch --tags", &dir);
    let versions = composer::lookup_versions(&dir);
    let (namespace, version, env, sandbox) = prompt_versions(&versions);
    let url = executor::deploy(&env, &namespace, &sandbox, &version).await;
    println!("Opening {}", &url);
    open::that(&url).unwrap();
}

pub async fn release_interactive() {
    let dir = u::root();
    u::sh("git fetch --tags", &dir);
    let versions = composer::lookup_versions(&dir);
    let namespace = prompt_names(&versions);
    let tag = tagger::next_tag(&namespace, "minor", "default");
    let url = executor::release(&namespace, "default", &tag.version).await;
    println!("Opening {}", &url);
    open::that(url).unwrap();
}

//

pub async fn build() {
    let dir = u::pwd();
    let maybe_function = composer::current_function(&dir);
    if let Some(f) = maybe_function {
        let rdir = &f.dir.strip_prefix(&format!("{}/", u::root())).unwrap();
        let namespace = u::second(&rdir, "/");
        let branch = tagger::git::branch_name(&dir);
        let url = executor::build(&namespace, &rdir, &branch).await;
        open::that(url).unwrap();
    } else {
        let rdir = &dir.strip_prefix(&format!("{}/", u::root())).unwrap();
        let namespace = u::second(&rdir, "/");
        let branch = &tagger::git::branch_name(&dir);
        let url = executor::build(&namespace, &rdir, &branch).await;
        println!("Opening {}", &url);
        open::that(url).unwrap();
    }
}

pub async fn release(service: Option<String>, suffix: Option<String>, unwind: bool) {
    let dir = u::pwd();
    let suffix = u::maybe_string(suffix, "default");
    let namespace = composer::topology_name(&dir);
    let service = u::maybe_string(service, &namespace);
    if unwind {
        tagger::unwind(&service);
    } else {
        let tag = tagger::next_tag(&service, "minor", &suffix);
        let url = executor::release(&service, &suffix, &tag.version).await;
        println!("Opening {}", &url);
        open::that(&url).unwrap();
    }
}

pub async fn deploy_snapshot(env: Option<String>, sandbox: Option<String>, snapshot: &str) {
    let env = match env {
        Some(e) => e,
        None => panic!("No env or profile specified"),
    };
    let sandbox = u::maybe_string(sandbox, "stable");
    let manifests = snapshotter::load(snapshot);

    for manifest in manifests {
        if !&manifest.version.is_empty() {
            println!(
                "Triggering CI build {}@{}.{}/{}",
                &manifest.namespace, &sandbox, &env, &manifest.version
            );
            executor::deploy(&env, &manifest.namespace, &sandbox, &manifest.version).await;
        }
    }
}

pub async fn deploy_pipeline(env: Option<String>, sandbox: Option<String>) {
    let env = match env {
        Some(e) => e,
        None => panic!("No env or profile specified"),
    };
    let sandbox = u::maybe_string(sandbox, "stable");
    let msg = format!(
        "This command will trigger a deploy to {}@{}. Do you want to continue?",
        &env, &sandbox
    );
    let ans = Confirm::new(&msg).with_default(false).prompt();

    let should_continue = match ans {
        Ok(true) => true,
        Ok(false) | Err(_) => false,
    };

    if should_continue {
        let url = executor::deploy_pipeline(&env, &sandbox).await;
        println!("Opening {}", &url);
        open::that(&url).unwrap();
    }
}

pub async fn deploy_version(
    topology: Option<String>,
    env: Option<String>,
    sandbox: Option<String>,
    version: &str,
) {
    let dir = u::pwd();
    let env = match env {
        Some(e) => e,
        None => panic!("No env or profile specified"),
    };
    let namespace = composer::topology_name(&dir);
    let name = u::maybe_string(topology, &namespace);
    let sandbox = u::maybe_string(sandbox, "stable");
    let version = if version == "latest" {
        composer::topology_version(&namespace)
    } else {
        version.to_string()
    };
    let url = executor::deploy(&env, &name, &sandbox, &version).await;
    println!("Opening {}", &url);
    open::that(&url).unwrap();
}

pub async fn deploy_branch(
    topology: Option<String>,
    env: Option<String>,
    sandbox: Option<String>,
    branch: &str,
) {
    let dir = u::pwd();
    let env = match env {
        Some(e) => e,
        None => panic!("No env or profile specified"),
    };
    let namespace = composer::topology_name(&dir);
    let name = u::maybe_string(topology, &namespace);
    let sandbox = u::maybe_string(sandbox, "stable");
    let url = executor::deploy_branch(&env, &name, &sandbox, branch).await;
    println!("Opening {}", &url);
    open::that(&url).unwrap();
}

pub async fn create(env: Option<String>, sandbox: Option<String>) {
    let env = match env {
        Some(e) => e,
        None => panic!("No env or profile specified"),
    };

    let dir = u::pwd();
    let rdir = &dir.strip_prefix(&format!("{}/", u::root())).unwrap();
    let sandbox = u::maybe_string(sandbox, "stable");
    let branch = u::sh("git rev-parse --abbrev-ref HEAD", &dir);
    let url = executor::create(&env, &sandbox, &rdir, &branch).await;
    println!("Opening {}", &url);
    open::that(&url).unwrap();
}

pub async fn update(env: Option<String>, sandbox: Option<String>) {
    let dir = u::pwd();
    let env = match env {
        Some(e) => e,
        None => panic!("No env or profile specified"),
    };

    let rdir = &dir.strip_prefix(&format!("{}/", u::root())).unwrap();
    let sandbox = u::maybe_string(sandbox, "stable");
    let branch = u::sh("git rev-parse --abbrev-ref HEAD", &dir);
    let url = executor::update(&env, &sandbox, &rdir, &branch).await;
    println!("Opening {}", &url);
    open::that(&url).unwrap();
}
