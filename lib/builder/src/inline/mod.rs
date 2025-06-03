mod python;
mod ruby;
mod rust;
mod node;
mod aws;

use authorizer::Auth;
use kit as u;
use kit::LogUpdate;
use std::io::stdout;
use kit::sh;

use compiler::{LangRuntime, Lang, Build};

fn gen_dockerfile(dir: &str, langr: &LangRuntime, force: bool) {
    match langr.to_lang() {
        Lang::Python => python::gen_dockerfile(dir, langr, force),
        Lang::Ruby => ruby::gen_dockerfile(dir),
        Lang::Rust => rust::gen_dockerfile(dir),
        Lang::Node => node::gen_dockerfile(dir),
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

async fn init() -> Auth {
    let config = compiler::config(&kit::pwd());
    let profile = config.aws.lambda.layers_profile.clone();
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            match profile.clone() {
                Some(p) => {
                    let role = config.ci.roles.get(&p).cloned();
                    Auth::new(profile.clone(), role).await
                },
                None => Auth::new(None, None).await
            }
        },
        Err(_) => {
            Auth::new(profile.clone(), None).await
        }
    }
}

async fn get_token() -> String {
    match std::env::var("CODEARTIFACT_AUTH_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            let auth = init().await;
            let client = aws::codeartifact::make_client(&auth).await;
            aws::codeartifact::get_auth_token(&client, &auth.name, &auth.account).await
        }
    }
}

async fn build_with_docker(dir: &str) {
    let root = &u::root();
    let token = get_token().await;
    println!("Found token: {}", &token);
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default={} -t {} --build-arg AUTH_TOKEN={} --build-context shared={root} .",
            &e,
            &token,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default  -t {} --build-arg AUTH_TOKEN={} --build-context shared={root} .",
            u::basedir(dir),
            &token
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

fn zip(dir: &str, langr: &LangRuntime) {
    match langr.to_lang() {
        Lang::Python => {
            let cmd = "rm -rf build && zip -q -9 -r ../../lambda.zip .";
            sh(&cmd, &format!("{}/build/python", dir));

        },
        Lang::Ruby => {
            let cmd = "cd build/ruby && find . -type d -name \".git\" | xargs rm -rf && rm -rf gems/3.2.0/cache/bundler/git && zip -q -9 --exclude=\"**/.git/**\" -r ../../lambda.zip . && cd -";
            sh(&cmd, dir);
        },
        Lang::Node => {
            let cmd = "cd build && zip -q -9 -r ../lambda.zip node_modules && cd -";
            sh(&cmd, dir);
        },
        Lang::Rust => {
            let command = "zip -q -r lambda.zip bootstrap";
            sh(command, dir);
        },
        _ => todo!()
    }
}


fn should_build_deps() -> bool {
    match std::env::var("TC_SKIP_BUILD") {
        Ok(_) => false,
        Err(_) => true
    }
}

pub async fn build(
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    bs: &Build
) -> String {

    let Build { command, force, .. } = bs;

    if should_build_deps() {

        sh("rm -rf lambda.zip deps.zip build", &dir);

        let mut log = LogUpdate::new(stdout()).unwrap();

        let _ = log.render(&format!("Building {name} - Generating Dockerfile"));
        gen_dockerfile(dir, langr, *force);
        gen_dockerignore(dir);

        let _ = log.render(&format!("Building {name} - Building with Docker"));
        build_with_docker(dir).await;

        let _ = log.render(&format!("Building {name} - Copying from container"));

        copy_from_docker(dir, langr);
        sh("rm -f Dockerfile wrapper .dockerignore", dir);

        let _ = log.render(&format!("Building {name} - Copying dependencies"));
        zip(dir, langr);

        sh("rm -rf build build.json", dir);
        sh(command, dir);

    } else {
        println!("Skipping Inline build");
        sh(command, dir);
    }
    format!("{}/lambda.zip", dir)
}
