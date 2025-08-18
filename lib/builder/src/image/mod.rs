mod aws_ecr;
mod python;

use crate::types::{
    BuildOutput,
    BuildStatus,
};
use authorizer::Auth;
use colored::Colorize;
use composer::{
    LangRuntime,
    spec::{
        ConfigSpec,
        ImageSpec,
        Lang,
    },
};
use kit as u;
use kit::sh;
use std::collections::HashMap;
use super::init_centralized_auth;

pub fn gen_base_dockerfile(dir: &str, runtime: &LangRuntime, commands: Vec<String>) {
    match runtime.to_lang() {
        Lang::Python => python::gen_base_dockerfile(dir, runtime, commands),
        _ => todo!(),
    }
}

pub fn gen_code_dockerfile(
    dir: &str,
    runtime: &LangRuntime,
    base_image: &str,
    commands: Vec<String>,
) {
    match runtime.to_lang() {
        Lang::Python => python::gen_code_dockerfile(dir, base_image, commands),
        _ => todo!(),
    }
}

fn build_with_docker(dir: &str, name: &str) -> (bool, String, String) {
    let root = &u::root();
    let cmd_str = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => format!(
            "docker buildx build --platform=linux/amd64 --provenance=false -t {} --build-context shared={root} .",
            name
        ),
        Err(_) => format!(
            "docker buildx build --ssh=default --platform=linux/amd64 --provenance=false --secret id=aws,src=$HOME/.aws/credentials -t {} --build-context shared={root} .",
            name
        ),
    };

    let (status, out, err) = u::runc(&cmd_str, dir);
    if !status {
        sh("rm -f Dockerfile wrapper", dir);
    }
    (status, out, err)
}

pub fn render_uri(uri: &str, repo: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("repo", repo);
    u::stencil(uri, table)
}

fn find_base_image_name(
    repo: &str,
    func_name: &str,
    images: &HashMap<String, ImageSpec>,
) -> String {
    let version = match images.get("base") {
        Some(b) => match &b.version {
            Some(v) => v,
            None => "latest",
        },
        None => "latest",
    };

    format!("{}/base:{}-{}", repo, func_name, version)
}

fn find_parent_image_name(
    repo: &str,
    func_name: &str,
    images: &HashMap<String, ImageSpec>,
    parent: Option<String>,
) -> String {
    let parent = u::maybe_string(parent, "base");
    match parent.as_ref() {
        "base" => find_base_image_name(repo, func_name, images),
        _ => render_uri(&parent, repo),
    }
}

pub async fn build(
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    images: &HashMap<String, ImageSpec>,
    image_kind: &str,
    uri: &str,
) -> BuildStatus {

    let auth = init_centralized_auth().await;
    aws_ecr::login(&auth, dir).await;

    let image_spec = match images.get(image_kind) {
        Some(p) => p,
        None => panic!("No image spec specified in build:images"),
    };

    let config = ConfigSpec::new(None);
    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };

    let image_dir = match &image_spec.dir {
        Some(d) => &d,
        None => dir,
    };


    let uri = render_uri(uri, repo);

    let bar = u::progress(3);

    let prefix = format!(
        "Building {} ({}/image/{})",
        name.blue(),
        langr.to_str(),
        image_kind
    );
    bar.set_prefix(prefix);

    match image_kind {
        "code" => {
            let parent_image_name =
                find_parent_image_name(repo, name, &images, image_spec.parent.clone());
            bar.inc(1);
            gen_code_dockerfile(
                image_dir,
                langr,
                &parent_image_name,
                image_spec.commands.clone(),
            );
            bar.inc(2);
            tracing::debug!("Building {} with parent {}", uri, &parent_image_name);
            let (status, out, err) = build_with_docker(image_dir, &uri);
            bar.inc(3);
            sh("rm -rf Dockerfile build build.json", image_dir);
            bar.finish();
            BuildStatus {
                path: uri.to_string(),
                status: status,
                out: out,
                err: err,
            }
        }
        "base" => {
            let base_image_name = find_base_image_name(repo, name, images);
            bar.inc(1);
            gen_base_dockerfile(image_dir, langr, image_spec.commands.clone());
            tracing::debug!(
                "Building image dir: {} name: {}",
                image_dir,
                &base_image_name
            );
            bar.inc(2);
            let (status, out, err) = build_with_docker(image_dir, &base_image_name);
            bar.inc(3);
            sh("rm -rf Dockerfile build build.json", image_dir);
            bar.finish();
            BuildStatus {
                path: base_image_name,
                status: status,
                out: out,
                err: err,
            }
        }
        _ => panic!("Invalid image kind"),
    }
}

pub async fn publish(auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, artifact, .. } = build;

    aws_ecr::login(&auth, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker push {}", &auth.name, artifact);
    u::run(&cmd, &dir);
}

pub async fn sync(auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, artifact, .. } = build;
    aws_ecr::login(auth, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker pull {}", &auth.name, artifact);
    u::run(&cmd, &dir);
}

pub async fn shell(auth: &Auth, dir: &str, uri: &str) {
    let config = ConfigSpec::new(None);
    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };
    aws_ecr::login(auth, &dir).await;
    let uri = render_uri(uri, repo);
    let cmd = format!("docker run --rm -it --entrypoint bash {}", uri);
    println!("{}", cmd);
    u::runcmd_stream(&cmd, dir);
}
