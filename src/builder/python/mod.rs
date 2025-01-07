pub mod advanced;
pub mod local;

use colored::Colorize;
use kit as u;

fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    u::runcmd_quiet(&clean, dir);
    u::sh(&run, dir);
    let id = u::sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    if id.is_empty() {
        println!("{}: ", dir);
        u::sh("rm -f requirements.txt Dockerfile", dir);
        std::panic::set_hook(Box::new(|_| {
            println!("Build failed");
        }));
        panic!("build failed")
    }
    u::sh(&format!("docker cp {}:/build build", id), dir);
    u::sh(&clean, dir);
}

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
}

pub fn zip(dir: &str, zipfile: &str) {
    if u::path_exists(dir, "build") {
        let cmd = format!("cd build && zip -q -9 -r ../{} . && cd -", zipfile);
        u::runcmd_quiet(&cmd, dir);
    }
}

fn extension_wrapper(name: &str) -> String {
    format!(
        r#"#!/bin/bash
set -euo pipefail

echo "{name}  launching extension"
exec "/opt/{name}/extension.py"

"#
    )
}

pub fn build_extension(dir: &str, name: &str) {
    u::sh("rm -rf *.zip build", dir);
    u::sh(&format!("mkdir -p build/{}", name), dir);
    u::sh(&format!("cp extension.py build/{}/", name), dir);

    u::mkdir("build/extensions");
    let wrapper_str = extension_wrapper(name);
    u::write_str(&format!("build/extensions/{}", name), &wrapper_str);
    u::sh("rm -f *.zip", dir);
    u::sh("cd build && zip -r -q ../extension.zip .", dir);
    u::sh("rm -rf build", dir);
    let size = size_of(dir, "extension.zip");
    println!("Size: {}", size);
}

fn copy(dir: &str) {
    if u::path_exists(dir, "src") {
        u::sh("cp -r src/* build/python/", dir);
    }
    if u::path_exists(dir, "lib") {
        u::sh("cp -r lib/* build/python/", dir);
    }
}

fn build_with_docker(
    dir: &str,
    name: &str,
    lang: &str,
    _kind: &str,
    deps_pre: Vec<String>,
    deps_post: Vec<String>,
    trace: bool,
) {
    let bar = kit::progress();
    let prefix = format!("Building {}", name.blue());
    bar.set_prefix(prefix);

    u::sh("rm -f deps.zip", dir);

    if u::path_exists(dir, "pyproject.toml") {
        bar.inc(10);
        advanced::gen_requirements_txt(dir, lang, trace);
    }

    bar.inc(30);
    advanced::gen_dockerfile(dir, lang, deps_pre, deps_post);

    advanced::build(dir, trace);

    bar.inc(50);

    copy_from_docker(dir);
    bar.inc(70);
    u::sh("rm -f Dockerfile", dir);
    if !u::path_exists(dir, "function.json") {
        copy(dir);
    }
    bar.inc(80);
    zip(dir, "deps.zip");

    bar.inc(100);
    let size = format!("({})", size_of(dir, "deps.zip").green());
    bar.set_message(size);
    bar.finish();
    //u::sh("rm -f requirements.txt", dir);
}

pub fn build_deps(
    dir: &str,
    lang: &str,
    name: &str,
    kind: &str,
    deps_pre: Vec<String>,
    deps_post: Vec<String>,
    no_docker: bool,
    trace: bool,
) {
    let pyp = format!("{}/pyproject.toml", dir);
    // FIXME: this is a hack pydantic does not build in docker
    if no_docker || u::file_contains(&pyp, "pydantic") {
        local::build(dir, name);
    } else {
        build_with_docker(dir, name, lang, kind, deps_pre, deps_post, trace);
    }
}

pub fn pack(dir: &str, command: &str) {
    u::sh("rm -f lambda.zip", dir);
    match command {
        "inline-deps" => {
            if u::path_exists(dir, "pyproject.toml") {
                local::build(dir, "inline-deps");
            }
            copy(dir);
            let cmd = "cd build/python && zip -q -9 -r ../../lambda.zip . && cd -";
            u::runcmd_quiet(&cmd, dir);
        }
        _ => {
            let c = format!(r"{}", command);
            u::sh(&c, dir);
        }
    }
}

pub fn clean(dir: &str) {
    u::sh("rm -rf lambda.zip dist __pycache__", dir);
}
