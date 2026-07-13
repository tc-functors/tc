use kit as u;
use kit::sh;

fn extension_wrapper(name: &str) -> String {
    format!(
        r#"#!/bin/bash
set -euo pipefail

echo "{name}  launching extension"
exec "/opt/{name}/extension.py"

"#
    )
}

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
}

pub fn gen_dockerfile(dir: &str) {
    let image = "public.ecr.aws/sam/build-python3.12:latest";

    let f = format!(
        r#"
FROM {image} AS intermediate

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY requirements.txt ./

RUN mkdir -p /build/python

RUN pip install -vvv -r requirements.txt --target=/build/python --upgrade

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    u::runcmd_quiet(&clean, dir);
    sh(&run, dir);
    let id = u::sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    if id.is_empty() {
        tracing::info!("{}: ", dir);
        sh("rm -f requirements.txt Dockerfile", dir);
        std::panic::set_hook(Box::new(|_| {
            tracing::error!("Build failed");
        }));
        panic!("build failed")
    }
    sh(&format!("docker cp {}:/build build_tmp", id), dir);
    sh(&clean, dir);
}

pub fn build_with_docker(dir: &str) -> (bool, String, String) {
    let cmd_str = format!(
        "docker build --no-cache  --platform=linux/amd64 . -t {}",
        u::basedir(dir)
        );
    let (status, out, err) = u::runc(&cmd_str, dir);
    if !status {
        sh("rm -rf Dockerfile build", dir);
        std::panic::set_hook(Box::new(|_| {
            println!("Build failed");
        }));
        panic!("Build failed")
    }
    (status, out, err)
}

fn install_deps(dir: &str) {
    gen_dockerfile(dir);
    let (_status, _out, _err) = build_with_docker(dir);
    copy_from_docker(dir);
}

pub fn build(dir: &str, name: &str) -> String {
    sh("rm -rf *.zip build", dir);
    sh(&format!("mkdir -p build/{}", name), dir);
    sh(&format!("mkdir -p build/python"), dir);
    install_deps(dir);
    sh(&format!("cp -r build_tmp/python/* build/python/"), dir);
    sh(&format!("cp extension.py build/{}/", name), dir);
    u::mkdir("build/extensions");
    let wrapper_str = extension_wrapper(name);
    u::write_str(&format!("build/extensions/{}", name), &wrapper_str);
    sh("rm -f *.zip", dir);
    sh("cd build && zip -r -q ../deps.zip .", dir);
    sh("rm -rf build build_tmp", dir);
    let size = size_of(dir, "deps.zip");
    println!("Size: {}", size);
    format!("{}/deps.zip", dir)
}
