mod python;
mod aws_ecr;

use compiler::LangRuntime;
use compiler::spec::{ImageSpec, ConfigSpec, Lang, BuildOutput};
use kit as u;
use kit::sh;
use std::collections::HashMap;
use authorizer::Auth;

pub fn gen_base_dockerfile(dir: &str, runtime: &LangRuntime, commands: Vec<String>) {
    match runtime.to_lang() {
        Lang::Python => python::gen_base_dockerfile(dir, runtime, commands),
        _ => todo!()
    }
}

pub fn gen_code_dockerfile(dir: &str, runtime: &LangRuntime, base_image: &str, commands: Vec<String>) {
    match runtime.to_lang() {
        Lang::Python => python::gen_code_dockerfile(dir, base_image, commands),
        _ => todo!()
    }
}

fn build_with_docker(dir: &str, name: &str) {
    let root = &u::root();
    let cmd_str = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => format!(
        "docker buildx build --platform=linux/amd64 --provenance=false -t {} --build-context shared={root} .",
        name
        ),
        Err(_) => format!(
        "docker buildx build --ssh=default --platform=linux/amd64 --provenance=false --secret id=aws,src=$HOME/.aws/credentials -t {} --build-context shared={root} .",
        name
        )
    };

    match std::env::var("TC_TRACE") {
        Ok(_) => u::runcmd_stream(&cmd_str, dir),
        Err(_) => {
            let status = u::runp(&cmd_str, dir);
            if !status {
                sh("rm -f Dockerfile wrapper", dir);
                panic!("Failed to build");
            }
        }
    }
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

pub fn build(
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    images: &HashMap<String, ImageSpec>,
    image_kind: &str,
    uri: &str
) -> String {

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

    match image_kind {
        "code" => {
            let parent_image_name =
                find_parent_image_name(
                    repo, name, &images, image_spec.parent.clone()
                );
            gen_code_dockerfile(
                image_dir, langr, &parent_image_name, image_spec.commands.clone()
            );
            tracing::debug!("Building {} with parent {}",
                            uri, &parent_image_name);
            build_with_docker(image_dir, &uri);
            sh("rm -rf Dockerfile build build.json", image_dir);
            uri.to_string()
        }
        "base" => {
            let base_image_name = find_base_image_name(repo, name, images);
            gen_base_dockerfile(
                image_dir, langr, image_spec.commands.clone()
            );
            tracing::debug!(
                "Building image dir: {} name: {}",
                image_dir,
                &base_image_name
            );
            build_with_docker(image_dir, &base_image_name);
            sh("rm -rf Dockerfile build build.json", image_dir);
            base_image_name
        }
        _ => panic!("Invalid image kind"),
    }
}


pub async fn publish(auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, artifact, .. } = build;
    aws_ecr::login(auth, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker push {}", &auth.name, artifact);
    u::run(&cmd, &dir);
}

pub async fn sync(auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, artifact, .. } = build;
    aws_ecr::login(auth, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker pull {}", &auth.name, artifact);
    u::run(&cmd, &dir);
}
