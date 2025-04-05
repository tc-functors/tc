use kit as u;
use kit::sh;

use super::LangRuntime;

fn find_image(runtime: &LangRuntime) -> String {
    match runtime {
        LangRuntime::Python312 => String::from("public.ecr.aws/sam/build-python3.12:latest"),
        _ => todo!()
    }
}

fn gen_dockerfile(dir: &str, runtime: &LangRuntime) {
    let image = find_image(&runtime);

    let f = format!(
            r#"
FROM {image} as intermediate

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

ENV PATH $HOME/.cargo/bin:$PATH

COPY slab.sh /slab.sh

RUN mkdir -p /build/lib

RUN dnf update -yy

RUN dnf -y install libXext libSM libXrender

RUN --mount=type=ssh /slab.sh

"#
        );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn build_with_docker(dir: &str) {
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!("docker build --no-cache  --ssh default={} --secret id=aws,src=$HOME/.aws/credentials . -t {}",
                         &e, u::basedir(dir)),
        Err(_) => format!("docker build --no-cache  --ssh default --secret id=aws,src=$HOME/.aws/credentials . -t {}",
                          u::basedir(dir))
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
    sh(&format!("docker cp {}:/build slab", id), dir);
    sh(&clean, dir);
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
    _name: &str,
    runtime: &LangRuntime,
    deps_pre: Vec<String>,
    deps_post: Vec<String>,
) -> String {

    if !u::path_exists(dir, "slab.sh") {
        panic!("No slab.sh found")
    }

    gen_dockerfile(dir, runtime);
    build_with_docker(dir);
    copy_from_docker(dir);
    sh("rm -f Dockerfile", dir);

    copy(dir);

    if u::path_exists(dir, "pyproject.toml") {
        sh("rm -f requirements.txt", dir);
    }
    format!("{}/slab", dir)
}
