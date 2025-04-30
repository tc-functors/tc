use super::version;
use kit as u;
use kit::*;
use std::collections::HashMap;

fn parent_tags_file(dir: &str) -> Option<String> {
    let paths = vec![
        u::absolutize(dir, "../tags.json"),
        u::absolutize(dir, "../../tags.json"),
        u::absolutize(dir, "../../../tags.json"),
        u::absolutize(dir, "../../../../tags.json"),
        s!("../tags.json"),
        s!("../../tags.json"),
        s!("../../../tags.json"),
        s!("../../../../tags.json"),
    ];
    u::any_path(paths)
}

fn load_tags(infra_dir: &str) -> HashMap<String, String> {
    let tags_file = format!("{}/tags.json", infra_dir);
    let parent_file = parent_tags_file(infra_dir);
    if u::file_exists(&tags_file) {
        let data: String = u::slurp(&tags_file);
        let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
        tags
    } else {
        match parent_file {
            Some(f) => {
                let data: String = u::slurp(&f);
                let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
                tags
            }
            None => HashMap::new(),
        }
    }
}

pub fn make(namespace: &str, infra_dir: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();

    let version = version::current_semver(namespace);
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), format!("{{{{sandbox}}}}"));
    h.insert(s!("version"), version);
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);

    let given_tags = load_tags(infra_dir);
    h.extend(given_tags);
    h
}
