mod python;
mod ruby;
mod rust;

use kit as u;
use kit::LogUpdate;
use std::io::stdout;
use kit::sh;

use compiler::{LangRuntime, Lang};

fn gen_dockerfile(dir: &str, langr: &LangRuntime) {
    match langr.to_lang() {
        Lang::Python => python::gen_dockerfile(dir, langr),
        Lang::Ruby => ruby::gen_dockerfile(dir),
        Lang::Rust => rust::gen_dockerfile(dir),
        _ => todo!()
    }
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

fn build_with_docker(dir: &str) {
    let root = &u::root();
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default={} -t {} --build-context shared={root} .",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default  -t {} --build-context shared={root} .",
            u::basedir(dir)
        ),
    };
    let status = u::runp(&cmd_str, dir);
    if !status {
        sh("rm -f Dockerfile wrapper", dir);
        panic!("Failed to build");
    }
}

fn copy_from_docker(dir: &str, langr: &LangRuntime) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    sh(&clean, dir);
    sh(&run, dir);
    let id = sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);

    match langr.to_lang() {
        Lang::Rust => {
            sh(&format!("docker cp {}:/build/target/lambda/bootstrap/bootstrap bootstrap", id), dir);
        },
        _ => {
            sh(&format!("docker cp {}:/build build", id), dir);
        }
    }



    sh(&clean, dir);
    sh("rm -f Dockerfile wrapper", dir);
}

pub fn build(
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    given_command: &str
) -> String {

    sh("rm -rf lambda.zip deps.zip build", &dir);

    let mut log = LogUpdate::new(stdout()).unwrap();

    let _ = log.render(&format!("Building {name} - Generating Dockerfile"));
    gen_dockerfile(dir, langr);
    gen_dockerignore(dir);

    let _ = log.render(&format!("Building {name} - Building with Docker"));
    build_with_docker(dir);

    let _ = log.render(&format!("Building {name} - Copying from container"));
    copy_from_docker(dir, langr);
    sh("rm -f Dockerfile wrapper .dockerignore", dir);

    let _ = log.render(&format!("Building {name} - Copying dependencies"));
    let cmd = "rm -rf build && zip -q -9 -r ../../lambda.zip .";
    sh(&cmd, &format!("{}/build/python", dir));
    sh("rm -rf build build.json", dir);
    sh(given_command, dir);
    format!("{}/lambda.zip", dir)
}
