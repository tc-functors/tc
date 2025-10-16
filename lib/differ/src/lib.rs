use composer::{
    Function,
    Topology,
};
use kit as u;
use kit::*;
use std::collections::HashMap;

fn files_modified_in_branch() -> Vec<String> {
    let dir = u::pwd();
    let cmd = "git rev-parse --abbrev-ref HEAD";
    let branch = sh(&cmd, &dir);
    if branch == "main" || branch == "HEAD" {
        vec![]
    } else {
        let out = u::sh(
            &format!(
                "git whatchanged --name-only --pretty=\"\" main..{} | xargs dirname | uniq",
                branch
            ),
            &dir,
        );
        tracing::debug!("{}", &out);
        u::split_lines(&out).iter().map(|v| v.to_string()).collect()
    }
}

fn files_modified_uncommitted() -> Vec<String> {
    let dir = u::pwd();
    let out = u::sh("git ls-files -m | xargs dirname | sort | uniq", &dir);
    u::split_lines(&out).iter().map(|v| v.to_string()).collect()
}

pub fn find_between_versions(namespace: &str, from: &str, to: &str) -> Vec<String> {
    let dir = pwd();
    let from_tag = format!("{}-{}", namespace, from);
    let to_tag = format!("{}-{}", namespace, to);
    sh(
        &format!(
            "git fetch origin +refs/tags/{}:refs/tags/{}",
            &to_tag, &to_tag
        ),
        &dir,
    );
    sh(
        &format!(
            "git fetch origin +refs/tags/{}:refs/tags/{}",
            &from_tag, &from_tag
        ),
        &dir,
    );
    let cmd = format!(
        r#"git diff {}...{} --name-only . | xargs dirname | sort | uniq"#,
        &from_tag, &to_tag
    );
    tracing::debug!("{}", &cmd);
    let out = sh(&cmd, &dir);

    let lines = kit::split_lines(&out);
    lines.iter().map(|s| s.to_string()).collect()
}

pub fn diff_fns(
    namespace: &str,
    from: &str,
    to: &str,
    fns: &HashMap<String, Function>,
) -> HashMap<String, Function> {
    let mut changed_fns: HashMap<String, Function> = HashMap::new();

    tracing::debug!("Diffing namespace {} from: {} to: {}", namespace, from, to);

    let lines = if to == from {
        vec![]
    } else {
        find_between_versions(namespace, from, to)
    };

    let fmod_1 = match std::env::var("CI") {
        Ok(_) => vec![],
        Err(_) => files_modified_uncommitted(),
    };
    let fmod_2 = files_modified_in_branch();

    for (name, f) in fns {
        let maybe_rdir = &f.dir.strip_prefix(&format!("{}/", &u::root()));
        if let Some(rdir) = maybe_rdir {
            for line in &lines {
                if line.starts_with(rdir) {
                    changed_fns.insert(name.to_string(), f.clone());
                }
            }

            let maybe_pdir = &f.dir.strip_prefix(&format!("{}/", u::pwd()));
            if let Some(pdir) = maybe_pdir {
                if fmod_1.contains(&pdir.to_string()) {
                    changed_fns.insert(name.to_string(), f.clone());
                }
            }
            if fmod_2.contains(&rdir.to_string()) {
                changed_fns.insert(name.to_string(), f.clone());
            }
        } else {
            for line in &lines {
                if line.ends_with(name) {
                    changed_fns.insert(name.to_string(), f.clone());
                }
            }
        }
    }
    changed_fns
}

pub fn diff(topology: &Topology, from: &str, to: &str) {
    let fns = diff_fns(&topology.namespace, &from, &to, &topology.functions);

    println!("Modified functions:");
    for (name, _) in fns {
        println!("  - {}", name);
    }
    for (_, node) in &topology.nodes {
        let fns = diff_fns(&topology.namespace, &from, &to, &node.functions);
        for (name, _) in fns {
            println!(" - {}/{}", node.namespace, name);
        }
    }

    println!("");
    println!("Changelog:");

    let f = format!("{}-{}", &topology.namespace, &from);
    let t = format!("{}-{}", &topology.namespace, &to);
    let changes = tagger::commits(&f, &t);
    println!("{}", changes);
}
