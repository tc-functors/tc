use kit as u;
use kit::sh;
use kit::*;

use compiler::spec::ImageSpec;
use configurator::Config;
use std::collections::HashMap;

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 1")
    }
}

fn copy_cmd(dir: &str) -> String {
    if u::path_exists(dir, "requirements.txt") {
        s!("COPY requirements.txt ./")
    } else {
        s!("COPY . /var/task")
    }
}

fn gen_dockerfile(dir: &str, image_uri: &str, commands: Vec<String>) {
    let commands = deps_str(commands);

    let cp_command = copy_cmd(dir);

    let f = format!(
            r#"
FROM {image_uri}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

{cp_command}

RUN mkdir -p /model

RUN --mount=type=ssh --mount=type=secret,id=aws,target=/root/.aws/credentials {commands}

CMD [ "handler.handler" ]

"#
        );
        let dockerfile = format!("{}/Dockerfile", dir);
        u::write_str(&dockerfile, &f);
}


fn build_with_docker(dir: &str, name: &str) {
    let cmd_str = format!(
        "docker buildx build --no-cache --ssh default --platform linux/amd64 --provenance=false --secret id=aws,src=$HOME/.aws/credentials -t {} .", name);
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


fn render_uri(uri: &str, repo: &str) -> String {
    println!("repo: {}", uri);
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("repo", repo);
    u::stencil(uri, table)
}

pub fn build(dir: &str, name: &str, image_kind: &str, images: HashMap<String, ImageSpec>) -> String {

    let image_spec = match images.get(image_kind) {
        Some(p) => p,
        None => panic!("No image spec specified in build:images")
    };

    let config = Config::new(None, "dev");
    let parent_uri = render_uri(&image_spec.parent, &config.aws.ecr.repo);
    let image_dir = match &image_spec.dir {
        Some(d) => &d,
        None => dir
    };

    if !u::path_exists(image_dir, "Dockerfile") {
        gen_dockerfile(image_dir, &parent_uri, image_spec.commands.clone());
    }
    let image_name = format!("{}:{}-{}-latest", &config.aws.ecr.repo, &name, image_kind);
    println!("Building image {}", &image_name);
    build_with_docker(image_dir, &image_name);
    sh("rm -rf build build.json", image_dir);
    format!("docker")
}
