use kit as u;
use kit::sh;
use super::LangRuntime;

use compiler::spec::ImageSpec;
use configurator::Config;
use std::collections::HashMap;


fn find_build_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Python310 => "python3.10:latest",
        LangRuntime::Python311 => "python3.11:latest",
        LangRuntime::Python312 => "python3.12:latest",
        _ => todo!()
    };
    format!("public.ecr.aws/sam/build-{}", &tag)
}

fn find_runtime_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Python310 => "python:3.10",
        LangRuntime::Python311 => "python:3.11",
        LangRuntime::Python312 => "python:3.12",
        _ => todo!()
    };
    format!("public.ecr.aws/lambda/{}", &tag)
}

fn gen_req_cmd(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        format!("pip install poetry && poetry self add poetry-plugin-export && poetry config virtualenvs.create false && poetry lock && poetry export --without-hashes --format=requirements.txt > requirements.txt")
    } else {
        format!("echo 0")
    }
}

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 0")
    }
}

fn gen_base_dockerfile(dir: &str, runtime: &LangRuntime, commands: Vec<String>) {
    let commands = deps_str(commands);

    let build_image = find_build_image(runtime);
    let runtime_image = find_runtime_image(runtime);

    let req_cmd = gen_req_cmd(dir);

    let f = format!(
            r#"
FROM {build_image} AS build-image

WORKDIR /var/task

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY . ./

RUN {req_cmd}

RUN mkdir -p /model

RUN --mount=type=ssh --mount=type=secret,id=aws,target=/root/.aws/credentials {commands}

FROM {runtime_image}

COPY --from=build-image /build/python /opt/python
COPY --from=build-image /model /model

"#
        );
        let dockerfile = format!("{}/Dockerfile", dir);
        u::write_str(&dockerfile, &f);
}


fn gen_code_dockerfile(dir: &str, base_image: &str, commands: Vec<String>) {
    let commands = deps_str(commands);
    let f = format!(
            r#"
FROM {base_image}

RUN {commands}

COPY . /var/task

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
     let mut table: HashMap<&str, &str> = HashMap::new();
     table.insert("repo", repo);
     u::stencil(uri, table)
 }

fn find_base_image_name(
    repo: &str,
    func_name: &str,
    images: &HashMap<String, ImageSpec>
) -> String {

    let version = match images.get("base") {
        Some(b) => match &b.version {
            Some(v) => v,
            None => "latest"
        },
        None => "latest"
    };

    format!("{}/base:{}-{}", repo, func_name, version)
}


pub fn build(
    dir: &str,
    name: &str,
    runtime: &LangRuntime,
    image_kind: &str,
    images: &HashMap<String, ImageSpec>,
    uri: &str,
) -> String {

    let image_spec = match images.get(image_kind) {
        Some(p) => p,
        None => panic!("No image spec specified in build:images")
    };

    let config = Config::new(None, "dev");
    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo
    };

    let image_dir = match &image_spec.dir {
        Some(d) => &d,
        None => dir
    };


    let base_image_name = find_base_image_name(repo, name, images);
    let uri = render_uri(uri, repo);

    match image_kind {
        "code" => {
            gen_code_dockerfile(
                image_dir,
                &base_image_name,
                image_spec.commands.clone(),

            );
            tracing::debug!("Building {} with base-image {}",
                            uri, &base_image_name);
            build_with_docker(image_dir, &uri);
            sh("rm -rf Dockerfile build build.json", image_dir);
            uri.to_string()
        }
        "base" => {
            gen_base_dockerfile(
                image_dir,
                runtime,
                image_spec.commands.clone()
            );
            tracing::debug!("Building image dir: {} name: {}",
                            image_dir, &base_image_name);
            build_with_docker(image_dir, &base_image_name);
            sh("rm -rf Dockerfile build build.json", image_dir);
            base_image_name
        },
        _ => panic!("Invalid image kind")
    }
}
