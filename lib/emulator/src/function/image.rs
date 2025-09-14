use provider::aws;
use provider::Auth;
use colored::Colorize;
use configurator::Config;
use composer::{
    Function,
};
use kit as u;
use kit::*;
use std::collections::HashMap;

fn gen_entry_point(lang: &str) -> String {
    match lang {
        "python3.12" | "python3.11" | "python3.10" | "python3.9" => format!(
            r"#!/bin/sh
exec /usr/local/bin/aws-lambda-rie /var/lang/bin/{lang} -m awslambdaric $@
"
        ),
        _ => s!(""),
    }
}

fn docker_build_cmd(name: &str, uri: &str) -> String {
    format!(
        r#"docker build -t build_{name} -f- . <<EOF
FROM {uri}
COPY ./entry_script.sh /entry_script.sh
RUN chmod +x /entry_script.sh
ENTRYPOINT [ "/entry_script.sh", "handler.handler" ]
EOF
"#,
    )
}

fn as_env_str(kvs: HashMap<String, String>) -> String {
    let mut s: String = String::from("");
    for (k, v) in kvs {
        let m = format!("-e {}={} ", &k, &v);
        s.push_str(&m);
    }
    s
}

async fn docker_run_cmd(auth: &Auth, name: &str, vars: &HashMap<String, String>) -> String {
    let env_str = as_env_str(vars.clone());
    let (key, secret, token) = auth.get_keys().await;
    format!(
        "docker run -p 9000:8080 -w /var/task -v $(pwd):/var/task {} -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e AWS_ACCESS_KEY_ID={} -e AWS_SECRET_ACCESS_KEY={} -e AWS_SESSION_TOKEN={} -e AWS_DEFAULT_REGION={} -e AWS_REGION={} -e PYTHONPATH=/opt/python:/var/runtime:/python:/python -e POWERTOOLS_METRICS_NAMESPACE=dev build_{name}",
        &env_str, &key, &secret, &token, &auth.region, &auth.region
    )
}

pub fn render_uri(uri: &str, repo: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("repo", repo);
    u::stencil(uri, table)
}

pub fn docker_shell_cmd(name: &str, region: &str, profile: &str, vars: &HashMap<String, String>) -> String {
    let env_str = as_env_str(vars.clone());
    format!(
            "docker run -p 8888:8888 -w /var/task -v $(pwd):/var/task {} -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e AWS_REGION={} -e AWS_PROFILE={} -v $HOME/.aws:/root/aws:ro -it --entrypoint /bin/bash build_{name}",
        &env_str, region, profile
    )
}

pub async fn run(auth: &Auth, dir: &str, function: &Function, shell: bool) {
    aws::ecr::login(auth, &dir).await;

    let Function { runtime, name, .. } = function;
    let lang = runtime.lang.to_str();

    let config = Config::new(None);

    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };

    let uri = &function.runtime.uri;
    let code_image_uri = render_uri(uri, repo);

    let entry = gen_entry_point(&lang);
    u::write_str("entry_script.sh", &entry);

    println!(
        "Building emulator: {} from {}",
        &name.cyan(),
        &code_image_uri
    );

    let b_cmd = docker_build_cmd(&name, &code_image_uri);
    u::sh(&b_cmd, dir);

    let vars = &function.runtime.environment;

    let cmd = if shell {
        docker_shell_cmd(&name, &auth.region, &auth.name, vars)
    } else {
        docker_run_cmd(&auth, &name, vars).await
    };
    u::runcmd_stream(&cmd, dir);
    u::sh("rm -f entry_script.sh", dir);
}

// shell
