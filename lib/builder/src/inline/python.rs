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

pub fn gen_dockerfile(dir: &str, runtime: &LangRuntime) {
    let pip_cmd = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => "pip install -r requirements.txt --target=/build/python --upgrade",
        Err(_) => {
            "pip install -r requirements.txt --platform manylinux2014_x86_64 --target=/build/python --implementation cp --only-binary=:all: --upgrade"
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

RUN {req_cmd}

RUN rm -rf /build/python && mkdir -p /build

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. {pip_cmd}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
