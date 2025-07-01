mod node;

use kit as u;
use kit::{
    sh,
};

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

fn build_with_docker(dir: &str) -> (bool, String, String) {
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
    let (status, out, err) = u::runc(&cmd_str, dir);
    if !status {
        sh("rm -f Dockerfile wrapper", dir);
    }
    (status, out, err)
}

fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));    sh(&clean, dir);
    sh(&run, dir);
    let id = sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);

    sh(&format!("docker cp {}:/build dist", id), dir);

    sh(&clean, dir);
    sh("rm -f Dockerfile wrapper", dir);
}

pub fn build(dir: &str, name: &str, command: &str) {
    let bar = u::progress(5);

    let prefix = format!("Building {} (node/page)", name);
    bar.set_prefix(prefix);
    node::gen_dockerfile(dir, command);

    bar.inc(1);
    gen_dockerignore(dir);
    bar.inc(2);

    let (status, out, err) = build_with_docker(dir);
    if !status {
        println!("{}", &out);
        println!("{}", &err);
    }

    bar.inc(3);

    copy_from_docker(dir);
    bar.inc(4);
    sh("rm -f Dockerfile wrapper .dockerignore", dir);
    bar.inc(5);

}
