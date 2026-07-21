use super::LangRuntime;
use kit as u;

fn find_image(runtime: &LangRuntime) -> String {
    match runtime {
        LangRuntime::Python310 => String::from("public.ecr.aws/sam/build-python3.10:latest"),
        LangRuntime::Python311 => String::from("public.ecr.aws/sam/build-python3.11:latest"),
        LangRuntime::Python312 => String::from("public.ecr.aws/sam/build-python3.12:latest"),
        LangRuntime::Python313 => String::from("public.ecr.aws/sam/build-python3.13:latest"),
        LangRuntime::Python314 => String::from("public.ecr.aws/sam/build-python3.14:latest"),
        _ => todo!(),
    }
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

fn make_copy_cmd(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        String::from("COPY pyproject.toml ./")
    } else if u::path_exists(dir, "requirements.txt") {
        String::from("COPY requirements.txt ./")
    } else {
        String::from("RUN echo 0")
    }
}

fn make_install_command(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        format!("uv sync --no-dev && uv pip install -r pyproject.toml --target=/build/python")
    } else if u::path_exists(dir, "requirements.txt") {
        format!("uv pip install -r requirements.txt --target=/build/python")
    } else {
        format!("RUN echo 0")
    }
}


pub fn gen_dockerfile(dir: &str, runtime: &LangRuntime, pre: &Vec<String>, post: &Vec<String>) {
    let pre = deps_str(pre.to_vec());
    let post = deps_str(post.to_vec());
    let install_cmd = make_install_command(dir);
    let cp_command = make_copy_cmd(dir);

    let build_context = &u::root();
    let image = find_image(&runtime);

    let f = format!(
        r#"
FROM {image} AS intermediate
WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
{cp_command}

COPY --from=shared . {build_context}/

RUN rm -rf /build/python && mkdir -p /build

RUN {pre}

RUN --mount=type=ssh --mount=type=cache,target=/.root/cache --mount=target=shared,type=bind,source=. {install_cmd}

RUN --mount=type=secret,id=aws-key,env=AWS_ACCESS_KEY_ID --mount=type=secret,id=aws-secret,env=AWS_SECRET_ACCESS_KEY --mount=type=secret,id=aws-session,env=AWS_SESSION_TOKEN {post}


"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn gen_dockerfile_unshared(
    dir: &str,
    runtime: &LangRuntime,
    pre: &Vec<String>,
    post: &Vec<String>,
) {
    let pre = deps_str(pre.to_vec());
    let post = deps_str(post.to_vec());
    let install_cmd = make_install_command(dir);
    let cp_command = make_copy_cmd(dir);
    let image = find_image(&runtime);

    let f = format!(
        r#"
FROM {image} AS intermediate
WORKDIR {dir}

{cp_command}

RUN rm -rf /build/python && mkdir -p /build

RUN {pre}

RUN --mount=type=cache,target=/.root/cache {install_cmd}

RUN {post}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
