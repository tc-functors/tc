mod aws_lambda;
mod python;
mod ruby;
use crate::types::{
    BuildOutput,
    BuildStatus,
};
use crate::Auth;
use colored::Colorize;
use composer::{
    Build,
    Lang,
    LangRuntime,
};
use kit as u;
use kit::sh;
use std::collections::HashMap;

fn should_split(dir: &str) -> bool {
    let zipfile = "deps.zip";
    let size;
    if u::path_exists(dir, zipfile) {
        size = u::path_size(dir, zipfile);
    } else {
        return false;
    }
    size >= 60000000.0
}

fn split(dir: &str) {
    let zipfile = format!("{}/deps.zip", dir);
    let size;
    if u::file_exists(&zipfile) {
        size = u::file_size(&zipfile);
    } else {
        panic!("No zip found");
    }
    if size >= 60000000.0 {
        let cmd = format!("zipsplit {} -n 50000000", zipfile);
        u::runcmd_stream(&cmd, dir);
    }
}

pub async fn do_publish(auth: &Auth, lang: &str, layer_name: &str, zipfile: &str) {
    println!("Using profile {}", auth.name);
    let client = aws_lambda::make_client(auth).await;

    if u::file_exists(zipfile) {
        println!("Publishing {}", layer_name.blue());
        let version = aws_lambda::publish(&client, layer_name, zipfile, lang).await;
        aws_lambda::add_permission(&client, layer_name, version).await;
        println!("(version: {})", version);
    }
}

async fn layer_arn(auth: &Auth, name: &str, version: Option<String>) -> String {
    match version {
        Some(v) => {
            let layer = format!("{}:{}", name, &v);
            auth.layer_arn(&layer)
        }
        None => {
            let client = aws_lambda::make_client(&auth).await;
            aws_lambda::find_layer_version(&client, name).await.unwrap()
        }
    }
}

pub async fn publish(auth: &Auth, build: &BuildOutput) {
    let BuildOutput {
        dir,
        runtime,
        name,
        artifact,
        ..
    } = build;

    let lang = runtime.to_str();
    if should_split(&dir) {
        println!("Split layer ... {}", &name);
        split(&dir);
        if u::path_exists(dir, "deps1.zip") {
            do_publish(auth, &lang, &format!("{}-0-dev", &name), "deps1.zip").await;
        }
        if u::path_exists(dir, "deps2.zip") {
            do_publish(auth, &lang, &format!("{}-1-dev", &name), "deps2.zip").await;
        }
        if u::path_exists(dir, "deps3.zip") {
            do_publish(auth, &lang, &format!("{}-2-dev", &name), "deps3.zip").await;
        }
    } else {
        let layer_name = format!("{}-dev", &name);
        do_publish(auth, &lang, &layer_name, &artifact).await;
    }
}

pub async fn promote(auth: &Auth, layer_name: &str, lang: &str, version: Option<String>) {
    let client = aws_lambda::make_client(&auth).await;
    let dev_layer_name = format!("{}-dev", layer_name);

    let dev_layer_arn = layer_arn(&auth, &dev_layer_name, version).await;
    println!("Promoting {}", dev_layer_arn);
    let maybe_url = aws_lambda::get_code_url(&client, &dev_layer_arn).await;

    match maybe_url {
        Some(url) => {
            let tmp_path = std::env::temp_dir();
            let tmp_dir = tmp_path.to_string_lossy();
            let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
            u::download(&url, HashMap::new(), &tmp_zip_file).await;

            let size = u::file_size(&tmp_zip_file);
            println!(
                "Publishing {} ({})",
                layer_name,
                u::file_size_human(size).green()
            );

            let version = aws_lambda::publish(&client, layer_name, &tmp_zip_file, lang).await;

            println!("Published {}:{} (stable)", layer_name, version);
            aws_lambda::add_permission(&client, layer_name, version).await;
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => panic!("Layer promotion failed"),
    }
}

fn gen_dockerfile(dir: &str, langr: &LangRuntime) {
    match langr.to_lang() {
        Lang::Python => python::gen_dockerfile(dir, langr),
        Lang::Ruby => ruby::gen_dockerfile(dir),
        _ => todo!(),
    }
}

fn copy_from_docker(dir: &str) {
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
    sh(&format!("docker cp {}:/build build", id), dir);
    sh(&clean, dir);
}

pub fn build_with_docker(dir: &str) -> (bool, String, String) {
    let root = &u::root();
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker build --no-cache  --platform=linux/amd64 --ssh default={} --secret id=aws,src=$HOME/.aws/credentials --build-context shared={root} . -t {}",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker build --no-cache  --platform=linux/amd64 --ssh default --secret id=aws,src=$HOME/.aws/credentials --build-context shared={root} . -t {}",
            u::basedir(dir)
        ),
    };
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

fn zip(dir: &str, langr: &LangRuntime) {
    match langr.to_lang() {
        Lang::Python => {
            let cmd = "rm -rf build && zip -q -9 -r ../deps.zip .";
            sh(&cmd, &format!("{}/build", dir));
        }
        Lang::Ruby => ruby::zip(dir, "deps.zip"),
        _ => (),
    }
}

fn copy(dir: &str, langr: &LangRuntime) {
    match langr.to_lang() {
        Lang::Ruby => ruby::copy(dir),
        _ => (),
    }
}

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
}

fn clean(dir: &str) {
    if u::path_exists(dir, "pyproject.toml") {
        sh("rm -f requirements.txt", dir);
    }
}

pub fn build(dir: &str, name: &str, langr: &LangRuntime, _bspec: &Build) -> BuildStatus {
    sh("rm -f deps.zip", dir);
    gen_dockerfile(dir, langr);
    let (status, out, err) = build_with_docker(dir);
    copy_from_docker(dir);
    if !u::path_exists(dir, "function.yml") && !u::path_exists(dir, "function.json") {
        copy(dir, langr);
    }
    zip(dir, langr);
    u::runcmd_quiet("rm -rf vendor && rm -rf bundler", dir);
    let size = format!("({})", size_of(dir, "deps.zip").green());
    println!("Size: {} {}", name, size);

    clean(dir);

    BuildStatus {
        path: format!("{}/deps.zip", dir),
        status: status,
        out: out,
        err: err,
    }
}
