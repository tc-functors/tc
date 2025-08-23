use super::LangRuntime;
use kit as u;

fn find_image(runtime: &LangRuntime) -> String {
    match runtime {
        LangRuntime::Python310 => String::from("public.ecr.aws/sam/build-python3.10:latest"),
        LangRuntime::Python311 => String::from("public.ecr.aws/sam/build-python3.11:latest"),
        LangRuntime::Python312 => String::from("public.ecr.aws/sam/build-python3.12:latest"),
        _ => todo!(),
    }
}

fn gen_req_cmd(dir: &str) -> String {
    if u::path_exists(dir, "pyproject.toml") {
        format!(
            "pip install poetry && poetry self add poetry-plugin-export && poetry config virtualenvs.create false && poetry lock && poetry export --without-hashes --format=requirements.txt > requirements.txt"
        )
    } else {
        format!("echo 1")
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

pub fn gen_dockerfile(dir: &str, runtime: &LangRuntime, pre: &Vec<String>, post: &Vec<String>) {
    let pre = deps_str(pre.to_vec());
    let post = deps_str(post.to_vec());
    let pip_cmd = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => "pip install -r requirements.txt --target=/build/python",
        Err(_) => {
            "pip install -r requirements.txt --platform manylinux2014_x86_64 --target=/build/python --implementation cp --only-binary=:all:"
        }
    };

    let build_context = &u::root();
    let image = find_image(&runtime);
    let req_cmd = gen_req_cmd(dir);

    let f = format!(
        r#"
FROM {image} AS intermediate
WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY pyproject.toml ./

COPY --from=shared . {build_context}/

RUN --mount=type=ssh {req_cmd}

RUN rm -rf /build/python && mkdir -p /build

RUN {pre}

RUN --mount=type=ssh --mount=type=cache,target=/.root/cache --mount=target=shared,type=bind,source=. {pip_cmd}

RUN --mount=type=ssh --mount=type=secret,id=aws,target=/root/.aws/credentials {post}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
