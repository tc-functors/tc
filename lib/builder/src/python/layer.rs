use super::LangRuntime;
use colored::Colorize;
use kit as u;
use kit::sh;

// FIXME: use ldd
fn shared_objects() -> Vec<&'static str> {
    vec![
        "cp -r /usr/lib64/libnghttp2.so.14.20.0 /build/lib/libnghttp2.so.14",
        "&& cp /usr/lib64/libcurl.so.4.8.0 /build/lib/libcurl.so.4",
        "&& cp /usr/lib64/libidn2.so.0.3.7 /build/lib/libidn2.so.0",
        "&& cp /usr/lib64/liblber-2.4.so.2.10.7 /build/lib/liblber-2.4.so.2",
        "&& cp /usr/lib64/libldap-2.4.so.2.10.7 /build/lib/libldap-2.4.so.2",
        "&& cp /usr/lib64/libnss3.so /build/lib/libnss3.so",
        "&& cp /usr/lib64/libsmime3.so /build/lib/libsmime3.so",
        "&& cp /usr/lib64/libssl3.so /build/lib/libssl3.so",
        "&& cp /usr/lib64/libunistring.so.0.1.2 /build/lib/libunistring.so.0",
        "&& cp /usr/lib64/libsasl2.so.3.0.0 /build/lib/libsasl2.so.3",
        "&& cp /usr/lib64/libssh2.so.1.0.1 /build/lib/libssh2.so.1",
        "&& cp --preserve=links /usr/lib64/libSM.so.6* /build/lib/",
        "&& cp --preserve=links /usr/lib64/libXrender.so.1* /build/lib/",
        "&& cp --preserve=links /usr/lib64/libXext.so.6* /build/lib/",
    ]
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

fn gen_dockerfile(dir: &str, runtime: &LangRuntime, pre: Vec<String>, post: Vec<String>) {
    let extra_str = u::vec_to_str(shared_objects());
    let extra_deps_pre = deps_str(pre);
    let extra_deps_post = deps_str(post);

    let pip_cmd = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => "pip install -r requirements.txt --target=/build/python --upgrade",
        Err(_) => {
            "pip install -r requirements.txt --platform manylinux2014_x86_64 --target=/build/python --implementation cp --only-binary=:all: --upgrade"
        }
    };

    let req_cmd = gen_req_cmd(dir);
    let image = find_image(&runtime);

    if runtime == &LangRuntime::Python312 {
        let f = format!(
            r#"
FROM {image} as intermediate

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY pyproject.toml ./

RUN {req_cmd}

ENV PATH $HOME/.cargo/bin:$PATH

RUN mkdir -p /build/lib

RUN dnf update -yy


RUN dnf -y install libXext libSM libXrender

RUN --mount=type=ssh pip install -vvv -r requirements.txt --target=/build/python --implementation cp --only-binary=:all: --upgrade

"#
        );
        let dockerfile = format!("{}/Dockerfile", dir);
        u::write_str(&dockerfile, &f);
    } else {

        let f = format!(
            r#"
FROM {image} as intermediate

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY pyproject.toml ./

RUN {req_cmd}

ENV PATH $HOME/.cargo/bin:$PATH

RUN mkdir -p /build/lib

RUN yum update -yy

RUN yum -y install libXext libSM libXrender

RUN {extra_deps_pre}

RUN {extra_str}

RUN --mount=type=ssh {pip_cmd}

RUN --mount=type=secret,id=aws,target=/root/.aws/credentials {extra_deps_post}

"#
        );
        let dockerfile = format!("{}/Dockerfile", dir);
        u::write_str(&dockerfile, &f);
    }
}


pub fn build_with_docker(dir: &str) {
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker build --no-cache  --platform=linux/amd64 --ssh default={} --secret id=aws,src=$HOME/.aws/credentials . -t {}",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker build --no-cache  --platform=linux/amd64 --ssh default --secret id=aws,src=$HOME/.aws/credentials . -t {}",
            u::basedir(dir)
        ),
    };
    let ret = u::runp(&cmd_str, dir);
    if !ret {
        sh("rm -rf Dockerfile build", dir);
        std::panic::set_hook(Box::new(|_| {
            println!("Build failed");
        }));
        panic!("Build failed")
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

fn copy(dir: &str) {
    if u::path_exists(dir, "src") {
        u::sh("cp -r src/* build/python/", dir);
    }
    if u::path_exists(dir, "lib") {
        u::sh("cp -r lib/* build/python/", dir);
    }
}

pub fn build(
    dir: &str,
    name: &str,
    runtime: &LangRuntime,
    deps_pre: Vec<String>,
    deps_post: Vec<String>,
) -> String {
    sh("rm -f deps.zip", dir);

    gen_dockerfile(dir, runtime, deps_pre, deps_post);
    build_with_docker(dir);
    copy_from_docker(dir);
    sh("rm -f Dockerfile", dir);

    let cmd = "rm -rf build && zip -q -9 -r ../deps.zip .";
    sh(&cmd, &format!("{}/build", dir));

    let size = format!("({})", size_of(dir, "deps.zip").green());
    println!("{} ({}", name, size);
    // if u::path_exists(dir, "pyproject.toml") {
    //     sh("rm -f requirements.txt", dir);
    // }
    //sh("rm -rf build", dir);
    format!("{}/deps.zip", dir)
}
