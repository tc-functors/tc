use super::Function;
use crate::spec::function::{
    FunctionSpec,
    LangRuntime,
};
use kit as u;
use kit::*;
use serde_derive::Serialize;
use std::collections::HashMap;
use walkdir::WalkDir;

pub fn guess_runtime(dir: &str) -> LangRuntime {
    let function = Function::new(dir, dir, "", "");
    function.runtime.lang
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
    pub runtime: LangRuntime,
    pub merge: bool,
    pub dirty: bool,
}

fn standalone_layer(dir: &str) -> Layer {
    Layer {
        kind: s!("implicit"),
        source: s!("standalone"),
        name: u::basedir(dir).to_string(),
        path: dir.to_string(),
        runtime: guess_runtime(dir),
        merge: true,
        dirty: is_dirty(dir),
    }
}

fn external_layers(dir: &str) -> Vec<Layer> {
    let fspec = FunctionSpec::new(dir);
    let mut layers: Vec<Layer> = vec![];
    if let Some(runtime) = fspec.runtime {
        for x in runtime.layers {
            let layer = Layer {
                kind: s!("external"),
                source: s!("function"),
                name: x,
                path: dir.to_string(),
                runtime: runtime.lang.to_owned(),
                merge: false,
                dirty: is_dirty(dir),
            };
            layers.push(layer);
        }
    }
    layers
}

fn function_layer(dir: &str) -> Layer {
    let fspec = Function::new(dir, dir, "", "");
    let name = match fspec.layer_name {
        Some(fln) => fln,
        None => u::basedir(dir).to_string(),
    };

    Layer {
        kind: s!("implicit"),
        source: s!("function"),
        name: name,
        path: dir.to_string(),
        runtime: fspec.runtime.lang,
        merge: false,
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

pub fn find(functions: HashMap<String, Function>) -> Vec<Layer> {
    let mut layers: Vec<Layer> = vec![];
    for (path, f) in functions {
        match f.layer_name {
            Some(name) => {
                if layerable(&path) {
                    let layer = Layer {
                        kind: s!("implicit"),
                        source: s!("topology"),
                        name: name,
                        path: path.to_owned(),
                        runtime: f.runtime.lang.to_owned(),
                        merge: false,
                        dirty: is_dirty(&path),
                    };
                    layers.push(layer);
                    let mut external = external_layers(&path);
                    layers.append(&mut external);
                }
            }
            None => (),
        }
    }
    layers.sort_by_key(|x| x.name.to_owned());
    layers
}
