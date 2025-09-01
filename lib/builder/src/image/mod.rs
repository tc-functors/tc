mod aws_ecr;
mod python;

use itertools::Itertools;
use crate::types::{
    BuildOutput,
    BuildStatus,
};

use authorizer::Auth;
use colored::Colorize;
use composer::{
    LangRuntime,
    Build,
    spec::{
        ConfigSpec,
        Lang,
        BuildKind
    },
};
use kit as u;
use kit::sh;
use std::collections::HashMap;

fn gen_base_dockerfile(
    dir: &str,
    runtime: &LangRuntime,
    pre: &Vec<String>,
    post: &Vec<String>
) {
    match runtime.to_lang() {
        Lang::Python => python::gen_base_dockerfile(dir, runtime, pre, post),
        _ => todo!(),
    }
}

fn gen_code_dockerfile(
    dir: &str,
    runtime: &LangRuntime,
    base_image_uri: &str,
) {
    match runtime.to_lang() {
        Lang::Python => python::gen_code_dockerfile(dir, base_image_uri),
        _ => todo!(),
    }
}

async fn build_with_docker(
    auth: &Auth,
    cont_name: &str,
    dir: &str,
    name: &str,
    code_only: bool,
) -> (bool, String, String) {

    let key_file = format!("/tmp/{}-key.txt", cont_name);
    let secret_file = format!("/tmp/{}-secret.txt", cont_name);
    let session_file = format!("/tmp/{}-session.txt", cont_name);

    let container_sha = format!("{}_{}", cont_name, u::checksum_str(dir));

    let create_cont_str = format!("docker buildx create --platform linux/amd64 --name {container_sha} --use --bootstrap");

    u::sh(&create_cont_str, dir);

    let cmd_str = if code_only {
        format!("docker build --platform=linux/amd64 -t {} .", name)
    } else {
        let root = &u::root();
        let (key, secret, token) = auth.get_keys().await;


        u::write_str(&key_file, &key);
        u::write_str(&secret_file, &secret);
        u::write_str(&session_file, &token);

        format!("docker buildx build --platform=linux/amd64 --ssh default --provenance=false --load -t {} --secret id=aws-key,src={} --secret id=aws-secret,src={} --secret id=aws-session,src={} --builder {container_sha} --build-context shared={root} .",
        name, &key_file, &secret_file, &session_file)
    };

    tracing::debug!("Building with docker {}", &cmd_str);

    let (status, out, err) = u::runc(&cmd_str, dir);

    if !status {
        sh("rm -f Dockerfile wrapper", dir);
    }
    sh(&format!("rm -f {}", &key_file), dir);
    sh(&format!("rm -f {}", &secret_file), dir);
    sh(&format!("rm -f {}", &session_file), dir);
    (status, out, err)
}

pub fn render_uri(uri: &str, repo: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("repo", repo);
    u::stencil(uri, table)
}

fn find_base_image_uri(uri: &str, version: Option<String>) -> String {
    tracing::debug!("Resolving base uri from {}", &uri);
    let (prefix, fname, _version) = uri.split("_").collect_tuple().unwrap();
    let version = u::maybe_string(version, "latest");
    format!("{}_{}_base_{}", prefix, fname, version)
}


async fn init(profile: Option<String>, assume_role: Option<String>) -> Auth {
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = match assume_role {
                Some(r) => Some(r),
                None => {
                    let config = composer::config(&kit::pwd());
                    let p = u::maybe_string(profile.clone(), "default");
                    config.ci.roles.get(&p).cloned()
                }
            };
            Auth::new(profile.clone(), role).await
        }
        Err(_) => Auth::new(profile.clone(), assume_role).await,
    }
}

pub async fn init_centralized_auth() -> Auth {
    let config = ConfigSpec::new(None);
    let profile = config.aws.lambda.layers_profile.clone();
    let auth = init(profile.clone(), None).await;
    let centralized = auth
        .assume(profile.clone(), config.role_to_assume(profile))
        .await;
    centralized
}


pub async fn build(
    _auth: &Auth,
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    uri: &str,
    bspec: &Build,
    code_only: bool
) -> BuildStatus {

    let Build { pre, post, version, .. } = bspec;

    let auth = init_centralized_auth().await;

    aws_ecr::login(&auth, dir).await;

    let config = ConfigSpec::new(None);

    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };

    let bar = u::progress(3);

    let prefix = format!(
        "Building {} ({}/image)",
        name.blue(),
        langr.to_str(),
    );

    bar.set_prefix(prefix);

    bar.inc(1);

    let code_image_uri = render_uri(uri, repo);
    let base_image_uri = find_base_image_uri(&code_image_uri, version.clone());

    if code_only {
        // let repo_ns = u::second(repo, "/");
        // let base_exists = aws_ecr::image_exists(&auth, &repo_ns, &base_image_uri).await;
        // if !base_exists {
        //    println!("Base image {} does not exist", &base_image_uri);
        // }
        gen_code_dockerfile(dir, langr, &base_image_uri);
    } else {
        gen_base_dockerfile(dir, langr, pre, post);
    }

    let uri = if code_only {
        code_image_uri
    } else {
        base_image_uri
    };

    let (status, out, err) = build_with_docker(&auth, name, dir, &uri, code_only).await;

    tracing::debug!("Building {}", uri);

    bar.inc(3);
    sh("rm -rf Dockerfile build build.json", dir);
    bar.finish();
    BuildStatus {
        path: uri.to_string(),
        status: status,
        out: out,
        err: err,
    }
}

pub async fn publish(_auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, artifact, .. } = build;

    let auth = init_centralized_auth().await;
    aws_ecr::login(&auth, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker push {}", &auth.name, artifact);
    u::run(&cmd, &dir);
}

pub async fn sync(auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, artifact, version, .. } = build;
    aws_ecr::login(auth, &dir).await;
    let base_image_uri = find_base_image_uri(&artifact, version.clone());
    println!("Pulling {}", &base_image_uri);
    let cmd = format!("AWS_PROFILE={} docker pull {}", &auth.name, &base_image_uri);
    u::run(&cmd, &dir);
    //let cmd = format!("AWS_PROFILE={} docker pull {}", &auth.name, artifact);
    //u::run(&cmd, &dir);
}

pub async fn shell(auth: &Auth, dir: &str, uri: &str, version: Option<String>, kind: BuildKind) {
    let config = ConfigSpec::new(None);
    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };
    aws_ecr::login(auth, &dir).await;
    let uri = render_uri(uri, repo);

    let uri = match kind {
        BuildKind::Image => find_base_image_uri(&uri, version.clone()),
        BuildKind::Code => uri,
        _ => uri
    };
    let cmd = format!("docker run --rm -it --entrypoint bash {}", uri);
    tracing::debug!("{}", cmd);

    u::runcmd_stream(&cmd, dir);
}
