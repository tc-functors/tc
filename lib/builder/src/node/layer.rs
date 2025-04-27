use colored::Colorize;
use compiler::LangRuntime;
use kit as u;
use kit::sh;

fn find_image(runtime: &LangRuntime) -> String {
    match runtime {
        LangRuntime::Node22 => String::from("public.ecr.aws/sam/build-nodejs22.x:latest"),
        LangRuntime::Node20 => String::from("public.ecr.aws/sam/build-nodejs20.x:latest"),
        _ => todo!(),
    }
}

fn gen_dockerfile(dir: &str, runtime: &LangRuntime) {
    let image = find_image(&runtime);

    let f = format!(
        r#"
FROM {image} as intermediate

COPY package.json ./

RUN mkdir -p /build/lib

RUN npm install --omit=dev

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn build_with_docker(dir: &str) {
    let cmd_str = format!("docker build --no-cache . -t {}", u::basedir(dir));
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
        u::sh("cp -r src/* build/nodejs/", dir);
    }
    if u::path_exists(dir, "lib") {
        u::sh("cp -r lib/* build/nodejs/", dir);
    }
}

pub fn build(
    dir: &str,
    name: &str,
    runtime: &LangRuntime,
    _deps_pre: Vec<String>,
    _deps_post: Vec<String>,
) -> String {
    sh("rm -f deps.zip", dir);
    gen_dockerfile(dir, runtime);
    build_with_docker(dir);
    copy_from_docker(dir);
    sh("rm -f Dockerfile", dir);

    if !u::path_exists(dir, "function.json") {
        copy(dir);
    }
    zip(dir, "deps.zip");
    let size = format!("({})", size_of(dir, "deps.zip").green());
    println!("{} ({}", name, size);
    sh("rm -rf build", dir);
    format!("{}/deps.zip", dir)
}
