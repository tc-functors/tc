use kit as u;

fn gen_req_cmd(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        format!(
            "pip install poetry && poetry self add poetry-plugin-export && poetry config virtualenvs.create false && poetry lock && poetry export --without-hashes --format=requirements.txt > requirements.txt && pip install -r requirements.txt --target=/opt/python"
        )
    } else if u::path_exists(dir, "requirements.txt") {
        format!("pip install -r requirements.txt --target=/opt/python")
    } else {
        format!("echo 0")
    }
}

fn make_cmd(handler: &str) -> String {
    let parts = handler.split(" ").collect::<Vec<&str>>();
    format!("{:?}", &parts)
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

pub fn gen_dockerfile(dir: &str, handler: &str, port: &i32, pre: &Vec<String>) {
    let cmd = make_cmd(handler);
    let pre = deps_str(pre.to_vec());
    let req_cmd = gen_req_cmd(dir);
    let f = format!(
        r#"
FROM public.ecr.aws/lambda/microvms:al2023-minimal
RUN dnf install -y python3 && dnf clean all
ENV PYTHONPATH="${{PYTHONPATH}}:/opt/python"
WORKDIR /app
COPY . .
RUN {req_cmd}
RUN {pre}
EXPOSE {port}
CMD {cmd}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
