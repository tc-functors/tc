use super::LangRuntime;
use kit as u;
use kit::{
    LogUpdate,
    sh,
};
use std::io::stdout;

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
}

fn gen_dockerignore(dir: &str) {
    let f = format!(r#"
**/node_modules/
**/dist
**/logs
**/target
**/vendor
**/build
.git
npm-debug.log
.coverage
.coverage.*
.env
.venv
.pyenv
**/.venv/
**/site-packages/
*.zip
"#);
    let file = format!("{}/.dockerignore", dir);
    u::write_str(&file, &f);
}

fn find_image(runtime: &LangRuntime) -> String {
    match runtime {
        LangRuntime::Python310 => String::from("public.ecr.aws/sam/build-python3.10:latest"),
        LangRuntime::Python311 => String::from("public.ecr.aws/sam/build-python3.11:latest"),
        LangRuntime::Python312 => String::from("public.ecr.aws/sam/build-python3.12:latest"),
        _ => todo!(),
    }
}

fn gen_req_cmd(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        format!(
            "pip install poetry && poetry self add poetry-plugin-export && poetry config virtualenvs.create false && poetry lock && poetry export --without-hashes --format=requirements.txt > requirements.txt"
        )
    } else {
        format!("echo 1")
    }
}

fn gen_dockerfile(dir: &str, runtime: &LangRuntime) {
    let pip_cmd = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => "pip install -r requirements.txt --target=/build/python --upgrade",
        Err(_) => {
            "pip install -r requirements.txt --platform manylinux2014_x86_64 --target=/build/python --implementation cp --only-binary=:all: --upgrade"
        }
    };

    let build_context = &top_level();
    let image = find_image(&runtime);
    let req_cmd = gen_req_cmd(dir);

    let f = format!(
        r#"
FROM {image} AS intermediate
WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY pyproject.toml ./

COPY --from=shared . {build_context}/

RUN {req_cmd}

RUN rm -rf /build/python && mkdir -p /build

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. {pip_cmd}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn build_with_docker(dir: &str) {
    let root = &top_level();
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --ssh default={} -t {} --build-context shared={root} .",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --ssh default  -t {} --build-context shared={root} .",
            u::basedir(dir)
        ),
    };
    let status = u::runp(&cmd_str, dir);
    if !status {
        sh("rm -f Dockerfile wrapper", dir);
        panic!("Failed to build");
    }
}

fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    sh(&clean, dir);
    sh(&run, dir);
    let id = sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);

    sh(&format!("docker cp {}:/build build", id), dir);
    sh(&clean, dir);
    sh("rm -f Dockerfile wrapper", dir);
}

// local

fn build_local(dir: &str, given_command: &str) {
    if u::path_exists(dir, "pyproject.toml") {
        sh("poetry config warnings.export false", dir);
        let cmd = "rm -f requirements.txt && poetry export -f requirements.txt --output requirements.txt --without-hashes --without dev";
        sh(cmd, dir);
    }
    let c = "pip install -r requirements.txt --platform manylinux2014_x86_64 --no-deps --upgrade --target build/python";
    sh(c, dir);
    let cmd = "zip -q -9 -r ../../lambda.zip . && rm -rf build/python";
    sh(&cmd, &format!("{}/build/python", dir));
    sh(given_command, dir);
    sh("rm -rf build build.json", dir);
}

fn build_docker(dir: &str, name: &str, runtime: &LangRuntime, given_command: &str) {
    let mut log = LogUpdate::new(stdout()).unwrap();

    let _ = log.render(&format!("Building {name} - Generating Dockerfile"));
    gen_dockerfile(dir, runtime);
    gen_dockerignore(dir);

    let _ = log.render(&format!("Building {name} - Building with Docker"));
    build_with_docker(dir);

    let _ = log.render(&format!("Building {name} - Copying from container"));
    copy_from_docker(dir);
    sh("rm -f Dockerfile wrapper .dockerignore", dir);

    let _ = log.render(&format!("Building {name} - Copying dependencies"));
    let cmd = "rm -rf build && zip -q -9 -r ../../lambda.zip .";
    sh(&cmd, &format!("{}/build/python", dir));
    sh("rm -rf build build.json", dir);
    sh(given_command, dir);
}

pub fn build(dir: &str, name: &str, runtime: &LangRuntime, given_command: &str) -> String {
    sh("rm -rf lambda.zip deps.zip build", dir);

    match std::env::var("TC_NO_DOCKER") {
        Ok(_) => build_local(dir, given_command),
        Err(_) => build_docker(dir, name, runtime, given_command),
    }
    format!("{}/lambda.zip", dir)
}
