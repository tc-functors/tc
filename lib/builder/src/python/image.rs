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
        LangRuntime::Python310 => "python3.10",
        LangRuntime::Python311 => "python3.11",
        LangRuntime::Python312 => "python3.12",
        _ => todo!()
    };
    format!("public.ecr.aws/lambda/{}", &tag)
}

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 1")
    }
}

fn gen_base_dockerfile(dir: &str, runtime: &LangRuntime, commands: Vec<String>) {
    let commands = deps_str(commands);

    let build_image = find_build_image(runtime);
    let runtime_image = find_runtime_image(runtime);

    let f = format!(
            r#"
FROM {build_image} AS build-image

WORKDIR /var/task

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY requirements.txt ./

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

fn find_image_name(repo: &str, image_kind: &str, func_name: &str) -> String {
    format!("{}/{}:{}-latest", repo, image_kind, func_name)
}

pub fn build(
    dir: &str,
    name: &str,
    runtime: &LangRuntime,
    image_kind: &str,
    images: HashMap<String, ImageSpec>
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

    //let parent_uri = render_uri(&image_spec.parent, repo);
    let image_dir = match &image_spec.dir {
        Some(d) => &d,
        None => dir
    };

    let base_image_name = find_image_name(repo, "base", name);

    match image_kind {
        "code" => {
            gen_code_dockerfile(
                image_dir,
                &base_image_name,
                image_spec.commands.clone()
            )
        }
        "base" => gen_base_dockerfile(
            image_dir,
            runtime,
            image_spec.commands.clone()
        ),
        _ => panic!("Invalid image kind")
    }
    let image_name = find_image_name(repo, image_kind, name);
    tracing::debug!("Building image dir: {} name: {}", image_dir, &image_name);
    build_with_docker(image_dir, &image_name);
    sh("rm -rf Dockerfile build build.json", image_dir);
    format!("docker")
}
