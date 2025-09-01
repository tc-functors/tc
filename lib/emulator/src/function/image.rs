use crate::aws;
use composer::{Function, ConfigSpec};
use kit::*;
use kit as u;
use authorizer::Auth;
use colored::Colorize;
use std::collections::HashMap;

fn gen_entry_point(lang: &str) -> String {
    match lang {
        "python3.10" | "python3.9" => format!(
            r"#!/bin/sh
exec /usr/local/bin/aws-lambda-rie /var/lang/bin/python3.10 -m awslambdaric $@
"
        ),
        _ => s!(""),
    }
}

fn docker_build_cmd(name: &str, uri: &str) -> String {
    format!(
        r#"docker build -t build_{name} -f- . <<EOF
FROM {uri}
RUN pip install boto3 -q -q -q --exists-action i
COPY ./entry_script.sh /entry_script.sh
RUN chmod +x /entry_script.sh
ENTRYPOINT [ "/entry_script.sh", "handler.handler" ]
EOF
"#,
    )
}

fn docker_run_cmd(name: &str) -> String {
    format!(
    "docker run -p 9000:8080 -v $(pwd)/build:/opt -w /var/task -v $(pwd):/var/task -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -v $HOME/.aws:/root/aws:ro -e AWS_REGION=us-west-2 -e PYTHONPATH=/opt/python:/var/runtime:/python:/python -e POWERTOOLS_METRICS_NAMESPACE=dev build_{name}"
    )
}

pub fn render_uri(uri: &str, repo: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("repo", repo);
    u::stencil(uri, table)
}

pub async fn run(auth: &Auth, dir: &str, function: &Function) {
    aws::ecr::login(auth, &dir).await;

    let Function { runtime, name, .. } = function;
    let lang = runtime.lang.to_str();

    let config = ConfigSpec::new(None);

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
        &name.cyan(), &code_image_uri
    );

    let b_cmd = docker_build_cmd(&name, &code_image_uri);
    u::sh(&b_cmd, dir);

    let cmd = docker_run_cmd(&name);
    u::runcmd_stream(&cmd, dir);
    u::sh("rm -f entry_script.sh", dir);
}
