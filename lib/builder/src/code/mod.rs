use crate::types::BuildStatus;
use crate::Auth;
use composer::{
    Build,
    Lang,
    spec::LangRuntime,
};
use kit as u;
use kit::sh;

fn find_build_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Python310 => "python3.10:latest",
        LangRuntime::Python311 => "python3.11:latest",
        LangRuntime::Python312 => "python3.12:latest",
        LangRuntime::Ruby32 => "ruby3.2:1.103.0-2023111622473",
        _ => todo!(),
    };
    format!("public.ecr.aws/sam/build-{}", &tag)
}

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 0")
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

fn gen_dockerfile(dir: &str, runtime: &LangRuntime, pre: &Vec<String>) {
    let pre_commands = deps_str(pre.to_vec());
    let build_image = find_build_image(runtime);
    let build_context = &u::root();

    let f = format!(
        r#"
FROM {build_image}

COPY --from=shared . {build_context}/

RUN mkdir -p /build

RUN --mount=type=ssh --mount=type=secret,id=aws-key,env=AWS_ACCESS_KEY_ID --mount=type=secret,id=aws-secret,env=AWS_SECRET_ACCESS_KEY --mount=type=secret,id=aws-session,env=AWS_SESSION_TOKEN --mount=target=shared,type=bind,source=. {pre_commands}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

async fn build_with_docker(auth: &Auth, name: &str, dir: &str) -> (bool, String, String) {
    let key_file = format!("/tmp/{}-key.txt", name);
    let secret_file = format!("/tmp/{}-secret.txt", name);
    let session_file = format!("/tmp/{}-session.txt", name);

    let root = &u::root();
    let (key, secret, token) = auth.get_keys().await;

    u::write_str(&key_file, &key);
    u::write_str(&secret_file, &secret);
    u::write_str(&session_file, &token);

    let cmd_str = format!(
        "docker buildx build --platform=linux/amd64 --provenance=false -t {} --secret id=aws-key,src={} --secret id=aws-secret,src={} --secret id=aws-session,src={} --build-context shared={root} .",
        name, &key_file, &secret_file, &session_file
    );

    tracing::debug!("Building with docker {}", &cmd_str);

    let (status, out, err) = u::runc(&cmd_str, dir);

    sh(&format!("rm -f {}", &key_file), dir);
    sh(&format!("rm -f {}", &secret_file), dir);
    sh(&format!("rm -f {}", &session_file), dir);

    if !status {
        sh("rm -f Dockerfile wrapper", dir);
        tracing::debug!("Build Fail {} {} {}", status, out, err);
        println!("Failed to build {}", name);
        std::process::exit(1);
    }

    (status, out, err)
}

fn copy_from_docker(dir: &str, name: &str) {
    let temp_cont = &format!("tmp-{}", name);
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, name);
    sh(&clean, dir);
    sh(&run, dir);
    let id = sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);
    if id.is_empty() {
        panic!("Cannot find source or build container");
    }
    sh(&format!("docker cp {}:/build build", id), dir);
    sh(&clean, dir);
    sh("rm -f Dockerfile wrapper", dir);
}

pub async fn build(
    auth: &Auth,
    dir: &str,
    name: &str,
    langr: &LangRuntime,
    spec: &Build,
) -> BuildStatus {
    match langr.to_lang() {
        Lang::Rust => super::inline::build(auth, dir, name, langr, spec).await,
        _ => {
            let Build { command, pre, .. } = spec;

            if !pre.is_empty() {
                gen_dockerignore(dir);
                gen_dockerfile(dir, langr, pre);

                let (status, out, err) = build_with_docker(&auth, name, dir).await;
                sh("rm -f Dockerfile wrapper .dockerignore", dir);
                if status {
                    copy_from_docker(dir, name);
                    let cmd = "zip -q -9 -r ../lambda.zip .";
                    let build_dir = format!("{}/build", dir);
                    sh(&cmd, &build_dir);
                    sh("rm -rf build build.json", dir);
                } else {
                    println!("Failed to run pre commands: {} {}", out, err);
                    std::process::exit(1)
                }
            }

            let c = format!(r"{}", command);
            let (status, out, err) = u::runc(&c, dir);
            BuildStatus {
                path: format!("{}/lambda.zip", dir),
                status: status,
                out: out,
                err: err,
            }
        }
    }
}
