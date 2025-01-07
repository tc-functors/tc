use aws::layer;
use aws::Env;
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

fn docker_cmd(name: &str, lang: &str, handler: &str) -> String {
    match lang {
        "rust" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/lambda/provided:al2
COPY bootstrap /var/runtime
CMD [ "{handler}" ]
EOF
"#
        ),
        "ruby3.2" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-ruby3.2:1.103.0-20231116224730
CMD [ "{handler}" ]
EOF
"#
        ),
        "python3.10" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-python3.10:latest
RUN pip install boto3 -q -q -q --exists-action i

CMD [ "{handler}" ]
EOF
"#
        ),
        "python3.11" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-python3.11:latest
RUN pip install boto3 -q -q -q --exists-action i

CMD [ "{handler}" ]
EOF
"#
        ),
        "python3.12" => format!(
            r#"docker build -t build_{name} -f- . <<EOF
FROM public.ecr.aws/sam/build-python3.12:latest
RUN pip install boto3 -q -q -q --exists-action i
CMD [ "{handler}" ]
EOF
"#
        ),
        _ => panic!("unknown..."),
    }
}

fn run_docker(name: &str, cmd: &str) {
    let dir = u::pwd();
    u::sh(cmd, &dir);
    let doc_start_cmd = format!("docker run --name {name} -p 9000:8080 -d build_{name}");
    u::sh(&doc_start_cmd, &dir);
}

fn get_shell_cmd(lang: &str, name: &str) -> String {
    match lang {
        "ruby3.2" => format!("docker run -v $(pwd)/build:/opt -v $(pwd):/var/task -e BUNDLE_CACHE_PATH=/opt/ruby/lib -e GEM_PATH=/opt/ruby/gems/3.2.0 -e RUBYLIB=/opt/ruby/lib -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e BUNDLE_GEMFILE=/opt/ruby/Gemfile -e AWS_REGION=us-west-2 -e Environment=dev -e POWERTOOLS_METRICS_NAMESPACE=dev -it --entrypoint /bin/bash build_{name}"),
        "python3.10" => format!("docker run -v $(pwd)/build:/opt -p 8888:8888 -w /var/task -v $(pwd):/var/task -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e AWS_REGION=us-west-2 -e Environment=dev -e PYTHONPATH=/opt/python:/var/runtime:/python:/python -e POWERTOOLS_METRICS_NAMESPACE=dev -it --entrypoint /bin/bash build_{name}"),
        "python3.11" => format!("docker run -v $(pwd)/build:/opt -p 8888:8888 -w /var/task -v $(pwd):/var/task -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e AWS_REGION=us-west-2 -e Environment=dev -e PYTHONPATH=/opt/python:/var/runtime:/python:/python -e POWERTOOLS_METRICS_NAMESPACE=dev -it --entrypoint /bin/bash build_{name}"),
    "python3.12" => format!("docker run -v $(pwd)/build:/opt -p 8888:8888 -w /var/task -v $(pwd):/var/task -e LD_LIBRARY_PATH=/usr/lib64:/opt/lib -e AWS_REGION=us-west-2 -e Environment=dev -e PYTHONPATH=/opt/python:/var/runtime:/python:/python -e POWERTOOLS_METRICS_NAMESPACE=dev -it --entrypoint /bin/bash build_{name}"),
        _ => s!("")
    }
}

pub async fn run(env: &Env, name: &str, lang: &str, handler: &str, layers: Vec<String>) {
    download_layers(env, layers).await;
    let cmd = docker_cmd(name, lang, handler);
    run_docker(name, &cmd);
    let exec_cmd = get_shell_cmd(lang, name);
    if !&exec_cmd.is_empty() {
        let dir = u::pwd();
        u::runcmd_stream(&exec_cmd, &dir);
        let stop_cmd = format!("docker rm {name}");
        u::sh(&stop_cmd, &dir);
    }
}
