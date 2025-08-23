use composer::LangRuntime;
use kit as u;

fn find_build_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Python310 => "python3.10:latest",
        LangRuntime::Python311 => "python3.11:latest",
        LangRuntime::Python312 => "python3.12:latest",
        _ => todo!(),
    };
    format!("public.ecr.aws/sam/build-{}", &tag)
}

fn find_runtime_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Python310 => "python:3.10",
        LangRuntime::Python311 => "python:3.11",
        LangRuntime::Python312 => "python:3.12",
        _ => todo!(),
    };
    format!("public.ecr.aws/lambda/{}", &tag)
}

fn gen_req_cmd(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        format!(
            "pip install poetry && poetry self add poetry-plugin-export && poetry config virtualenvs.create false && poetry lock && poetry export --without-hashes --format=requirements.txt > requirements.txt"
        )
    } else {
        format!("echo 0")
    }
}

fn deps_str(deps: &Vec<String>) -> String {
    let s = if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 0")
    };
    s.replace("AWS_PROFILE=cicd", "")
}

pub fn gen_base_dockerfile(
    dir: &str,
    runtime: &LangRuntime,
    pre: &Vec<String>,
    post: &Vec<String>
) {
    let pre_commands = deps_str(pre);
    let post_commands = deps_str(post);

    let build_image = find_build_image(runtime);
    let runtime_image = find_runtime_image(runtime);

    let req_cmd = gen_req_cmd(dir);
    let pip_cmd = "pip install -vv -r requirements.txt --target /build/python";

    let build_context = &u::root();

    let f = format!(
        r#"
FROM {build_image} AS build-image

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY . ./

COPY --from=shared . {build_context}/

RUN {pre_commands}

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. {req_cmd}

RUN mkdir -p /model

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. {pip_cmd}

RUN --mount=type=secret,id=aws-key,env=AWS_ACCESS_KEY_ID --mount=type=secret,id=aws-secret,env=AWS_SECRET_ACCESS_KEY --mount=type=secret,id=aws-session,env=AWS_SESSION_TOKEN {post_commands}

FROM {runtime_image} AS runtime

COPY --from=build-image /build/python /opt/python
COPY --from=build-image /model /model

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn gen_code_dockerfile(dir: &str, base_image: &str) {
    let f = format!(
        r#"
FROM {base_image}

ENV PATH=$PATH:/model/bin
ENV LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/model/lib

COPY . /var/task

CMD [ "handler.handler" ]
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
