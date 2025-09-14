mod node;
mod python;
mod ruby;
mod rust;

use crate::types::BuildStatus;
use colored::Colorize;
use composer::{
    Build,
    Lang,
    LangRuntime,
};
use kit as u;
use kit::sh;
use provider::{
    Auth,
    aws,
};

fn gen_dockerfile(
    dir: &str,
    langr: &LangRuntime,
    pre: &Vec<String>,
    post: &Vec<String>,
    skip_dev_deps: bool,
) {
    match langr.to_lang() {
        Lang::Python => python::gen_dockerfile(dir, langr, pre, post),
        Lang::Ruby => {
            if skip_dev_deps {
                ruby::gen_dockerfile(dir, pre, post);
            } else {
                ruby::gen_dockerfile_no_wrap(dir, pre, post);
            }
        }
        Lang::Rust => rust::gen_dockerfile(dir),
        Lang::Node => node::gen_dockerfile(dir),
        _ => todo!(),
    }
}

fn gen_dockerfile_unshared(
    dir: &str,
    langr: &LangRuntime,
    pre: &Vec<String>,
    post: &Vec<String>,
    _wrap: bool,
) {
    match langr.to_lang() {
        Lang::Python => python::gen_dockerfile_unshared(dir, langr, pre, post),
        Lang::Ruby => ruby::gen_dockerfile_unshared(dir, pre, post),
        Lang::Rust => rust::gen_dockerfile(dir),
        Lang::Node => node::gen_dockerfile(dir),
        _ => todo!(),
    }
}

fn gen_dockerignore(dir: &str) {
    let f = format!(
        r#"
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
"#
    );
    let file = format!("{}/.dockerignore", dir);
    u::write_str(&file, &f);
}

async fn get_token(auth: &Auth) -> String {
    match std::env::var("CODEARTIFACT_AUTH_TOKEN") {
        Ok(t) => t,
        Err(_) => {
            let client = aws::codeartifact::make_client(auth).await;
            aws::codeartifact::get_auth_token(&client, &auth.name, &auth.account).await
        }
    }
}

fn _create_buildx_container(name: &str, dir: &str) -> String {
    let container_sha = format!("{}_{}", name, u::checksum_str(dir));

    let create_cont_str = format!(
        "docker buildx create --platform linux/amd64 --name {container_sha} --use --bootstrap"
    );
    u::sh(&create_cont_str, dir);
    container_sha
}

async fn build_with_docker(
    auth: &Auth,
    dir: &str,
    langr: &LangRuntime,
    _name: &str,
    shared_context: bool,
) -> (bool, String, String) {
    let root = &u::root();
    let token = match langr.to_lang() {
        Lang::Node => get_token(auth).await,
        _ => String::from(""),
    };
    //let container_sha = create_buildx_container(name, dir);

    let cmd_str = if shared_context {
        format!(
            "docker buildx build --platform=linux/amd64 --ssh default --load -t {} --build-arg AUTH_TOKEN={} --build-context shared={root} .",
            u::basedir(dir),
            &token
        )
    } else {
        format!(
            "docker buildx build --platform=linux/amd64 -t {} .",
            u::basedir(dir)
        )
    };

    let (status, out, err) = u::runc(&cmd_str, dir);
    if !status {
        println!("Build failed: {} {}", out, err);
        sh("rm -f Dockerfile wrapper", dir);
        panic!("Build failed");
    }
    (status, out, err)
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
            sh(
                &format!(
                    "docker cp {}:/build/target/lambda/bootstrap/bootstrap bootstrap",
                    id
                ),
                dir,
            );
        }
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
        }
        Lang::Ruby => {
            let cmd = "cd build/ruby && find . -type d -name \".git\" | xargs rm -rf && rm -rf gems/3.2.0/cache/bundler/git && sed -i \"/group/,/end:/d\" Gemfile && zip -q -9 --exclude=\"**/.git/**\" -r ../../lambda.zip . && cd -";
            sh(&cmd, dir);
        }
        Lang::Node => {
            let cmd = "cd build && zip -q -9 -r ../lambda.zip node_modules && cd -";
            sh(&cmd, dir);
        }
        Lang::Rust => {
            let command = "zip -q -r lambda.zip bootstrap";
            sh(command, dir);
        }
        _ => todo!(),
    }
}

fn should_build_deps() -> bool {
    match std::env::var("TC_SKIP_BUILD") {
        Ok(_) => false,
        Err(_) => true,
    }
}

pub async fn build(
    auth: &Auth,
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    bs: &Build,
) -> BuildStatus {
    let Build {
        command,
        pre,
        post,
        shared_context,
        skip_dev_deps,
        ..
    } = bs;

    if should_build_deps() {
        sh("rm -rf lambda.zip deps.zip build", &dir);

        let bar = u::progress(8);

        let prefix = format!("Building {} ({}/inline)", name.blue(), langr.to_str());
        bar.set_prefix(prefix);

        if *shared_context {
            gen_dockerfile(dir, langr, pre, post, *skip_dev_deps)
        } else {
            gen_dockerfile_unshared(dir, langr, pre, post, *skip_dev_deps);
        }
        bar.inc(1);
        gen_dockerignore(dir);
        bar.inc(2);

        let (status, out, err) = build_with_docker(auth, dir, langr, name, *shared_context).await;
        bar.inc(3);

        if !status {
            println!("Inline build failed {}", name);
            println!("Err {}, {}", out, err);
            std::process::exit(1);
        }

        copy_from_docker(dir, langr);
        bar.inc(4);
        sh("rm -f Dockerfile wrapper .dockerignore", dir);
        bar.inc(5);

        zip(dir, langr);
        bar.inc(6);

        sh(command, dir);
        bar.inc(7);
        match std::env::var("TC_INSPECT_BUILD") {
            Ok(_) => (),
            Err(_) => {
                sh("rm -rf build build.json", dir);
            }
        }

        bar.inc(8);
        bar.finish();

        BuildStatus {
            path: format!("{}/lambda.zip", dir),
            status: status,
            out: out,
            err: err,
        }
    } else {
        println!("Skipping Inline build");
        sh(command, dir);
        BuildStatus {
            path: format!("{}/lambda.zip", dir),
            status: true,
            out: String::from(""),
            err: String::from(""),
        }
    }
}
