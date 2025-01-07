use aws::layer;
use aws::Env;
use colored::Colorize;
use kit as u;
use kit::*;
use std::collections::HashMap;
use std::env;

async fn download(url: &str, target_dir: &str) {
    let tmp_path = env::temp_dir();
    let tmp_dir = tmp_path.to_string_lossy();
    let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
    u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
    u::download(&url, HashMap::new(), &tmp_zip_file).await;
    u::sh(
        &format!("unzip -o {} -d {}", tmp_zip_file, target_dir),
        &tmp_dir,
    );
    u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
}

async fn download_layers(env: &Env, layers: Vec<String>) {
    let client = layer::make_client(env).await;
    let resolved_layers = env.resolve_layers(layers).await;
    let target_dir = format!("{}/build", &u::pwd());
    u::sh(&format!("rm -rf {}", &target_dir), &u::pwd());
    for layer in resolved_layers {
        println!("Fetching layer: {}", &layer);
        let maybe_url = layer::get_code_url(&client, &layer).await;
        match maybe_url {
            Some(url) => download(&url, &target_dir).await,
            None => (),
        }
    }
}

fn gen_entry_point(lang: &str) -> String {
    match lang {
        "python3.10" | "python3.9" => format!(
            r"#!/bin/sh
exec /usr/local/bin/aws-lambda-rie /var/lang/bin/python3.10 -m awslambdaric $@
"
        ),
        "ruby3.2" => {
            format!(
                r"#!/bin/sh
exec /usr/local/bin/aws-lambda-rie aws_lambda_ric $@ $@
"
            )
        }
        _ => s!(""),
    }
}

fn docker_build_cmd(name: &str, lang: &str, handler: &str) -> String {
    match lang {
        "ruby3.2" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-ruby3.2:1.103.0-20231116224730
COPY ./entry_script.sh /entry_script.sh
RUN chmod +x /entry_script.sh
ENTRYPOINT [ "/entry_script.sh","{handler}" ]
EOF
"#
        ),
        "python3.10" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-python3.10:latest
RUN pip install boto3 -q -q -q --exists-action i
COPY ./entry_script.sh /entry_script.sh
RUN chmod +x /entry_script.sh
ENTRYPOINT [ "/entry_script.sh", "{handler}" ]
EOF
"#
        ),

        "python3.12" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-python3.12:latest
RUN pip install boto3 -q -q -q --exists-action i
COPY ./entry_script.sh /entry_script.sh
RUN chmod +x /entry_script.sh
ENTRYPOINT [ "/entry_script.sh", "{handler}" ]
EOF
"#
        ),

        "rust" | "janet" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/lambda/provided:al2023
COPY bootstrap /var/runtime
CMD [ "{handler}" ]
EOF
"#
        ),
        _ => panic!("unknown..."),
    }
}

fn docker_run_cmd(name: &str, lang: &str) -> String {
    let env = match std::env::var("AWS_PROFILE") {
        Ok(e) => e,
        Err(_) => s!("dev"),
    };
    match lang {
        "ruby3.2" => format!("docker run -p 9000:8080 -v $(pwd)/build:/opt -v $(pwd):/var/task -e BUNDLE_CACHE_PATH=/opt/ruby/lib -e GEM_PATH=/opt/ruby/gems/3.2.0 -v $HOME/.aws:/root/aws:ro -e RUBYLIB=/opt/ruby/lib -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e BUNDLE_GEMFILE=/opt/ruby/Gemfile -e AWS_REGION=us-west-2 -e Environment={env} -e AWS_PROFILE={env} -e POWERTOOLS_METRICS_NAMESPACE=dev build_{name}"),

       "python3.10" => format!("docker run -p 9000:8080 -v $(pwd)/build:/opt -w /var/task -v $(pwd):/var/task -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -v $HOME/.aws:/root/aws:ro -e AWS_REGION=us-west-2 -e Environment={env} -e AWS_PROFILE={env} -e PYTHONPATH=/opt/python:/var/runtime:/python:/python -e POWERTOOLS_METRICS_NAMESPACE=dev build_{name}"),

        "rust" | "janet" => format!("docker run --rm -p 9000:8080 -e AWS_REGION=us-west-2 -e Environment=dev build_{name}"),

        _ => s!("")
    }
}

pub async fn run(env: &Env, name: &str, lang: &str, layers: Vec<String>, handler: &str) {
    download_layers(env, layers).await;
    let dir = u::pwd();

    let entry = gen_entry_point(lang);
    u::write_str("entry_script.sh", &entry);

    println!(
        "Building emulator: {} ({}) {}",
        &name.cyan(),
        lang,
        handler.green()
    );
    let b_cmd = docker_build_cmd(name, lang, handler);
    u::sh(&b_cmd, &dir);

    let cmd = docker_run_cmd(name, lang);
    u::runcmd_stream(&cmd, &dir);
    u::sh("rm -f entry_script.sh", &dir);
}
