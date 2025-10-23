mod node;

use kit as u;
use kit::sh;
use provider::{
    Auth,
    aws,
};


async fn get_token(auth: &Auth) -> String {
    match std::env::var("TC_USE_CODEARTIFACT") {
        Ok(_) => {
            let auth = provider::init_centralized_auth(auth).await;
            let client = aws::codeartifact::make_client(&auth).await;
            aws::codeartifact::get_auth_token(&client, &auth.name, &auth.account).await
        },
        Err(_) => String::from("")
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

async fn build_with_docker(auth: &Auth, dir: &str) -> (bool, String, String) {
    let root = &u::root();

    let token = get_token(auth).await;

    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default={} --build-arg AUTH_TOKEN={} -t {} --build-context shared={root} .",
            &e,
            &token,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default --build-arg AUTH_TOKEN={} --load -t {} --build-context shared={root} .",
            &token,
            u::basedir(dir)
        ),
    };
    let (status, out, err) = u::runc(&cmd_str, dir);
    if !status {
        sh("rm -f Dockerfile wrapper", dir);
    }
    (status, out, err)
}

fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    sh(&clean, dir);
    sh(&run, dir);
    let id = sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);

    sh("rm -rf dist", dir);
    sh(&format!("docker cp {}:/build/dist dist", id), dir);

    sh(&clean, dir);
    sh("rm -f Dockerfile wrapper", dir);
}

fn clean_tmp(dir: &str, ct: &Option<String>) {
    if let Some(p) = ct {
        let cmd = format!("rm -f {}_tmp", &p);
        sh(&cmd, dir);
    }
}

pub async fn build(auth: &Auth, dir: &str, name: &str, command: &str, config_template: &Option<String>) {
    let bar = u::progress(5);

    let prefix = format!("Building {} (node/page)", name);
    bar.set_prefix(prefix);
    node::gen_dockerfile(dir, command, config_template);

    bar.inc(1);
    gen_dockerignore(dir);
    bar.inc(2);

    let (status, out, err) = build_with_docker(auth, dir).await;
    if !status {
        println!("{}", &out);
        println!("{}", &err);
        std::process::exit(1);
    }

    bar.inc(3);

    copy_from_docker(dir);
    bar.inc(4);
    sh("rm -f Dockerfile wrapper .dockerignore", dir);
    clean_tmp(dir, config_template);
    bar.inc(5);
}
