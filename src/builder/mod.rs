pub mod go;
pub mod janet;
pub mod python;
pub mod ruby;
pub mod rust;

use crate::compiler;
use crate::compiler::Layer;
use glob::glob;
use kit as u;
use kit::*;
use log::info;
use serde_derive::Serialize;

pub fn pack(lang: &str, dir: &str, task: &str) {
    match lang {
        "ruby3.2" | "ruby" | "ruby2.7" => ruby::pack(dir, task),
        "python3.7" | "python3.10" | "python3.11" | "python3.12" | "python3.9" => {
            python::pack(dir, task)
        }
        "go" => go::pack(dir, task),
        "rust" => rust::pack(dir),
        "janet" => janet::pack(dir),
        _ => (),
    }
}

pub fn pack_all(dir: &str) {
    let topology = compiler::compile(dir, false);
    for (path, function) in topology.functions {
        let lang = function.runtime.lang;
        let mtask = &function.tasks.get("build");
        match mtask {
            Some(task) => {
                pack(&lang, &path, &task);
            }
            _ => {
                pack(&lang, &path, "zip -9 lambda.zip *.rb *.py");
            }
        }
    }
}

pub fn clean_dir(dir: &str) {
    info!("Cleaning {}", u::dir_of(dir));
    ruby::clean(dir);
    python::clean(dir);
    rust::clean(dir);
    janet::clean(dir);
}

pub fn merge_dirs(dirs: Vec<String>)  {
    let cwd = u::pwd();
    let zipfile = format!("{}/deps.zip", &cwd);
    let build_dir = format!("{}/build", &cwd);
    u::sh(&format!("mkdir -p {}/ruby/lib", &build_dir), &cwd);
    for dir in dirs {
        if u::path_exists(&dir, "lib") && !&dir.ends_with("layer") && !&dir.ends_with("build") {
            u::sh(&format!("cp -r lib/* {}/ruby/lib/", &build_dir), &dir);
            u::sh(&format!("cp -r lib/* {}/lib/", &build_dir), &dir);
        }
    }
    let cmd = format!("zip -9 -q -r {} .", zipfile);
    u::sh(&cmd, &build_dir);
    let size = u::path_size(&cwd, "deps.zip");
    u::sh(&format!("rm -rf {}", &build_dir), &cwd);
    println!("Merged deps ({})", u::file_size_human(size));
    println!("To publish, run `tc publish --name NAME`")
}

pub fn merge(name: &str, layers: Vec<Layer>) -> Vec<BuildOutput> {
    let dir = u::pwd();
    let zipfile = format!("{}/deps.zip", &dir);
    let mut lang: String = u::empty();
    for layer in layers {
        let cmd = format!("zip -x \"ruby/lib/*/build/*\" -9 -q -r {} .", zipfile);
        let build_dir = format!("{}/build", &layer.path);
        u::sh(&cmd, &build_dir);
        lang = layer.lang;
    }
    let size = u::path_size(&dir, "deps.zip");
    info!("Merged deps ({})", u::file_size_human(size));
    let out = BuildOutput {
        name: name.to_string(),
        lang: lang,
        target: s!("layer"),
        dir: dir,
        zipfile: zipfile,
    };
    vec![out]
}

fn is_recursive(dir: &str) -> bool {
    let layers = compiler::find_layers();
    compiler::is_topology_dir(dir) || layers.len() > 0
}

fn should_merge(dir: &str, mergeable: &Vec<Layer>) -> bool {
    !compiler::is_topology_dir(dir) && mergeable.len() > 0
}

fn should_split(dir: &str) -> bool {
    let zipfile = "deps.zip";
    let size;
    if u::path_exists(dir, zipfile) {
        size = u::path_size(dir, zipfile);
    } else {
        return false;
    }
    size >= 70000000.0
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildOutput {
    pub name: String,
    pub lang: String,
    pub target: String,
    pub zipfile: String,
    pub dir: String,
}

fn split(dir: &str, name: &str, lang: &str) -> Vec<BuildOutput> {
    let zipfile = format!("{}/deps.zip", dir);
    let size;
    if u::file_exists(&zipfile) {
        size = u::file_size(&zipfile);
    } else {
        panic!("No zip found");
    }
    if size >= 70000000.0 {
        let cmd = format!("zipsplit {} -n 50000000", zipfile);
        u::runcmd_stream(&cmd, dir);
    }
    let zips = glob("deps*.zip").expect("Failed to read glob pattern");
    let mut outs: Vec<BuildOutput> = vec![];
    for (n, entry) in zips.enumerate() {
        match entry {
            Ok(z) => {
                if &z.to_string_lossy() != &zipfile && n != 0 {
                    let zname = format!("{}-{}", name, n);
                    let out = BuildOutput {
                        name: zname,
                        dir: dir.to_string(),
                        lang: lang.to_string(),
                        target: s!("layer"),
                        zipfile: z.to_string_lossy().to_string(),
                    };
                    outs.push(out);
                }
            }
            Err(_e) => (),
        }
    }
    outs
}

pub fn determine_kind(kind: Option<String>) -> String {
    match kind {
        Some(k) => k,
        _ => {
            if u::file_exists("extension.py") {
                s!("extension")
            } else {
                s!("deps")
            }
        }
    }
}

fn build_runtime(dir: &str, _name: &str, lang: &str, trace: bool) {
    match lang {
        "janet" => janet::build_runtime(dir, trace),
        _ => (),
    }
}

pub fn just_build_out(dir: &str, name: &str, lang: &str, target: &str) -> Vec<BuildOutput> {
    let zipfile = format!("{}/deps.zip", dir);
    let out = BuildOutput {
        name: name.to_owned(),
        dir: dir.to_string(),
        lang: lang.to_owned(),
        target: target.to_owned(),
        zipfile: zipfile,
    };
    vec![out]
}

fn build_deps(
    dir: &str,
    name: &str,
    lang: &str,
    kind: &str,
    deps_pre: Vec<String>,
    deps_post: Vec<String>,
    no_docker: bool,
    trace: bool,
) -> Vec<BuildOutput> {
    u::sh("rm -f *.zip", dir);
    u::sh("rm -rf build", dir);
    match lang {
        "ruby3.2" | "ruby" | "ruby2.7" => ruby::build_deps(dir, name, no_docker, trace),
        "python3.7" | "python3.9" | "python3.10" | "python3.11" | "python3.12" => {
            python::build_deps(dir, lang, name, kind, deps_pre, deps_post, no_docker, trace);
        }
        "rust" => rust::build_deps(dir, no_docker, trace),
        "janet" => janet::build_deps(dir, trace),
        _ => (),
    }
    if should_split(dir) && kind != "efs" {
        split(dir, name, lang)
    } else {
        let zipfile = format!("{}/deps.zip", dir);
        let out = BuildOutput {
            name: name.to_owned(),
            dir: dir.to_string(),
            lang: lang.to_owned(),
            target: kind.to_owned(),
            zipfile: zipfile,
        };
        vec![out]
    }
}

fn build_extension(dir: &str, lang: &str, name: &str) {
    u::sh("rm -f extension.zip", dir);
    match lang {
        "ruby3.2" | "ruby" | "ruby2.7" => (),
        "python3.10" => python::build_extension(dir, name),
        "rust" => rust::build_extension(dir),
        _ => python::build_extension(dir, name),
    }
}

fn build_code(dir: &str, _lang: &str) {
    u::sh("rm -f deps.zip build", &dir);
    let dirs = u::list_dir(dir);
    u::runcmd_quiet("mkdir -p build/ruby/lib", &dir);
    for d in dirs {
        if !d.ends_with("build") {
            let cmd = format!("cp -r {}/lib/* build/lib/", &d);
            u::runcmd_stream(&cmd, &dir);
        }
    }
    u::runcmd_quiet("cd build && zip -q -9 -r ../deps.zip .", &dir);
    let size = u::path_size(dir, "deps.zip");
    println!("Merged layer ({})", u::file_size_human(size));
}

pub fn mergeable_layers(layers: Vec<Layer>) -> Vec<Layer> {
    let mut m: Vec<Layer> = vec![];
    for layer in layers {
        if layer.merge {
            m.push(layer);
        }
    }
    m
}

fn should_build(layer: &Layer, dirty: bool) -> bool {
    if dirty {
        layer.dirty
    } else {
        &layer.kind == "implicit" || &layer.kind == "default"
    }
}

fn build_recursive(
    dir: &str,
    name: &str,
    no_docker: bool,
    trace: bool,
    dirty: bool,
) -> Vec<BuildOutput> {
    let mut outs: Vec<BuildOutput> = vec![];
    let layers = compiler::find_layers();
    for layer in layers.clone() {
        if should_build(&layer, dirty) {
            let mut out = build_deps(
                &layer.path,
                &layer.name,
                &layer.lang,
                &layer.target,
                layer.extra_deps_pre,
                layer.extra_deps_post,
                no_docker,
                trace,
            );
            outs.append(&mut out)
        }
    }
    let mergeable = mergeable_layers(layers);
    if should_merge(dir, &mergeable) {
        info!("Merging layers... {}", &mergeable.len());
        merge(name, mergeable)
    } else {
        outs
    }
}

pub async fn build(
    dir: &str,
    name: &str,
    kind: &str,
    no_docker: bool,
    trace: bool,
    dirty: bool,
) -> Vec<BuildOutput> {
    match kind {
        "code" => {
            let lang = &compiler::guess_lang(dir);
            build_code(dir, lang);
            let out = BuildOutput {
                name: u::basename(dir),
                dir: dir.to_string(),
                lang: lang.to_string(),
                target: s!("layer"),
                zipfile: format!("{}/deps.zip", dir),
            };
            vec![out]
        }
        "deps" => {
            if is_recursive(dir) {
                build_recursive(dir, name, no_docker, trace, dirty)
            } else {
                let lang = &compiler::guess_lang(dir);
                let target = compiler::determine_target(dir);
                build_deps(dir, name, lang, &target, vec![], vec![], no_docker, trace)
            }
        }
        "extension" => {
            let lang = &compiler::guess_lang(dir);
            build_extension(dir, &lang, name);
            let out = BuildOutput {
                name: u::basename(dir),
                dir: dir.to_string(),
                lang: lang.to_string(),
                target: s!("layer"),
                zipfile: format!("{}/extension.zip", dir),
            };
            vec![out]
        }
        "runtime" => {
            let lang = &compiler::guess_lang(dir);
            build_runtime(dir, name, &lang, trace);
            vec![]
        }
        _ => vec![],
    }
}

pub fn clean(dir: &str) {
    if is_recursive(dir) {
        let layers = compiler::find_layers();
        for layer in layers {
            clean_dir(&layer.path);
        }
    } else {
        clean_dir(dir)
    }
}
