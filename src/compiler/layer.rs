use serde_derive::Serialize;
use std::collections::HashMap;
use walkdir::WalkDir;

use super::FunctionSpec;
use kit as u;
use kit::*;

pub fn guess_lang(dir: &str) -> String {
    let function = FunctionSpec::new(dir, dir, "", "");
    match function {
        Some(f) => f.runtime.lang,
        None => s!("python3.10"),
    }
}

pub fn layerable(dir: &str) -> bool {
    if u::path_exists(dir, "function.json") {
        u::path_exists(dir, "Gemfile")
            || u::path_exists(dir, "pyproject.toml")
            || u::path_exists(dir, "requirements.txt")
            || u::path_exists(dir, "Cargo.toml")
    } else {
        false
    }
}

pub fn discoverable(dir: &str) -> bool {
    u::path_exists(dir, "Gemfile")
        || u::path_exists(dir, "pyproject.toml")
        || u::path_exists(dir, "requirements.txt")
        || u::path_exists(dir, "Cargo.toml")
}

fn files_modified() -> Vec<String> {
    match std::env::var("CIRCLE_SHA1") {
        Ok(sha) => {
            let s = format!("git diff --name-only {}^1", sha);
            let dir = u::pwd();
            let out = u::sh(&s, &dir);
            u::split_lines(&out)
                .iter()
                .map(|v| u::absolutize(&dir, v))
                .collect()
        }
        Err(_) => {
            let dir = u::pwd();
            let out = u::sh("git ls-files -m", &dir);
            u::split_lines(&out)
                .iter()
                .map(|v| u::absolutize(&dir, v))
                .collect()
        }
    }
}

fn is_dirty(dir: &str) -> bool {
    let modified = files_modified();
    modified.contains(&u::path_of(dir, "function.json"))
        || modified.contains(&u::path_of(dir, "Gemfile"))
        || modified.contains(&u::path_of(dir, "pyproject.toml"))
        || modified.contains(&u::path_of(dir, "requirements.txt"))
        || modified.contains(&u::path_of(dir, "Cargo.toml"))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Layer {
    pub kind: String,
    pub source: String,
    pub name: String,
    pub path: String,
    pub lang: String,
    pub merge: bool,
    pub target: String,
    pub extra_deps_pre: Vec<String>,
    pub extra_deps_post: Vec<String>,
    pub dirty: bool,
}

fn standalone_layer(dir: &str) -> Layer {
    Layer {
        kind: s!("implicit"),
        source: s!("standalone"),
        name: u::basedir(dir).to_string(),
        path: dir.to_string(),
        lang: guess_lang(dir),
        target: s!("layer"),
        merge: true,
        extra_deps_pre: vec![],
        extra_deps_post: vec![],
        dirty: is_dirty(dir),
    }
}

fn external_layers(dir: &str) -> Vec<Layer> {
    let fspec = FunctionSpec::parse(dir);
    let mut layers: Vec<Layer> = vec![];
    let xs = fspec.runtime.layers;
    for x in xs {
        let layer = Layer {
            kind: s!("external"),
            source: s!("function"),
            name: x,
            path: dir.to_string(),
            lang: fspec.runtime.lang.to_owned(),
            target: s!("layer"),
            merge: false,
            extra_deps_pre: vec![],
            extra_deps_post: vec![],
            dirty: is_dirty(dir),
        };
        layers.push(layer);
    }
    layers
}

fn function_layer(dir: &str) -> Layer {
    let fspec = FunctionSpec::new(dir, dir, "", "").unwrap();
    let name = match fspec.layer_name {
        Some(fln) => fln,
        None => u::basedir(dir).to_string(),
    };
    let target = match fspec.assets.get("MODEL_PATH") {
        Some(_) => "efs",
        _ => "layer",
    };
    let extra_deps_pre = match fspec.assets.get("EXTRA_DEPS_PRE") {
        Some(v) => serde_json::from_value(v.clone()).unwrap(),
        _ => vec![],
    };
    let extra_deps_post = match fspec.assets.get("EXTRA_DEPS_POST") {
        Some(v) => serde_json::from_value(v.clone()).unwrap(),
        _ => vec![],
    };
    Layer {
        kind: s!("implicit"),
        source: s!("function"),
        name: name,
        path: dir.to_string(),
        lang: fspec.runtime.lang,
        target: s!(target),
        merge: false,
        extra_deps_pre: extra_deps_pre,
        extra_deps_post: extra_deps_post,
        dirty: is_dirty(dir),
    }
}

pub fn discover() -> Vec<Layer> {
    let mut layers: Vec<Layer> = vec![];
    let dir = u::pwd();
    for entry in WalkDir::new(dir.clone())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_dir()
                && !e.path().to_string_lossy().contains("/build/ruby/")
                && !e.path().to_string_lossy().contains("/build/python/")
                && !e.path().to_string_lossy().contains("/target/")
                && !e.path().to_string_lossy().contains("/build/")
                && !e.path().to_string_lossy().contains("/vendor/")
                && !e.path().to_string_lossy().contains(".venv")
        })
    {
        let p = entry.path().to_string_lossy();
        if discoverable(&p) {
            if u::path_exists(&p, "function.json") {
                let layer = function_layer(&p);
                layers.push(layer);
                let mut external = external_layers(&p);
                layers.append(&mut external);
            } else {
                let layer = standalone_layer(&p);
                layers.push(layer)
            }
        }
    }
    layers.sort_by_key(|x| x.name.to_owned());
    layers
}

pub fn find(functions: HashMap<String, FunctionSpec>) -> Vec<Layer> {
    let mut layers: Vec<Layer> = vec![];
    for (path, f) in functions {
        let target = match f.assets.get("DEPS_PATH") {
            Some(_) => "efs",
            _ => "layer",
        };
        let extra_deps_pre = match f.assets.get("EXTRA_DEPS_PRE") {
            Some(v) => serde_json::from_value(v.clone()).unwrap(),
            _ => vec![],
        };
        let extra_deps_post = match f.assets.get("EXTRA_DEPS_POST") {
            Some(v) => serde_json::from_value(v.clone()).unwrap(),
            _ => vec![],
        };

        match target {
            "layer" => match f.layer_name {
                Some(name) => {
                    if layerable(&path) {
                        let layer = Layer {
                            kind: s!("implicit"),
                            source: s!("topology"),
                            name: name,
                            path: path.to_owned(),
                            lang: f.runtime.lang.to_owned(),
                            target: s!(target),
                            merge: false,
                            extra_deps_pre: vec![],
                            extra_deps_post: vec![],
                            dirty: is_dirty(&path),
                        };
                        layers.push(layer);
                        let mut external = external_layers(&path);
                        layers.append(&mut external);
                    }
                }
                None => (),
            },

            "efs" => {
                let layer = Layer {
                    kind: s!("default"),
                    source: s!("topology"),
                    name: u::basedir(&path).to_string(),
                    path: path.clone(),
                    lang: f.runtime.lang,
                    target: "efs".to_string(),
                    merge: false,
                    extra_deps_pre: extra_deps_pre,
                    extra_deps_post: extra_deps_post,
                    dirty: is_dirty(&path),
                };
                layers.push(layer);
            }

            _ => (),
        }
    }
    layers.sort_by_key(|x| x.name.to_owned());
    layers
}
