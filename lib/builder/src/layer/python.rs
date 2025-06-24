use super::LangRuntime;
use kit as u;

// FIXME: use ldd
fn shared_objects() -> Vec<&'static str> {
    vec![
        "cp -r /usr/lib64/libnghttp2.so.14.20.0 /build/lib/libnghttp2.so.14",
        "&& cp /usr/lib64/libcurl.so.4.8.0 /build/lib/libcurl.so.4",
        "&& cp /usr/lib64/libidn2.so.0.3.7 /build/lib/libidn2.so.0",
        "&& cp /usr/lib64/liblber-2.4.so.2.10.7 /build/lib/liblber-2.4.so.2",
        "&& cp /usr/lib64/libldap-2.4.so.2.10.7 /build/lib/libldap-2.4.so.2",
        "&& cp /usr/lib64/libnss3.so /build/lib/libnss3.so",
        "&& cp /usr/lib64/libsmime3.so /build/lib/libsmime3.so",
        "&& cp /usr/lib64/libssl3.so /build/lib/libssl3.so",
        "&& cp /usr/lib64/libunistring.so.0.1.2 /build/lib/libunistring.so.0",
        "&& cp /usr/lib64/libsasl2.so.3.0.0 /build/lib/libsasl2.so.3",
        "&& cp /usr/lib64/libssh2.so.1.0.1 /build/lib/libssh2.so.1",
    ]
}

fn find_image(runtime: &LangRuntime) -> String {
    match runtime {
        LangRuntime::Python310 => String::from("public.ecr.aws/sam/build-python3.10:latest"),
        LangRuntime::Python311 => String::from("public.ecr.aws/sam/build-python3.11:latest"),
        LangRuntime::Python312 => String::from("public.ecr.aws/sam/build-python3.12:latest"),
        LangRuntime::Python313 => String::from("public.ecr.aws/sam/build-python3.13:latest"),
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
    let _extra_str = u::vec_to_str(shared_objects());

    let _pip_cmd = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => "pip install -r requirements.txt --target=/build/python --upgrade",
        Err(_) => {
            "pip install -r requirements.txt --platform manylinux2014_x86_64 --target=/build/python --implementation cp --only-binary=:all: --upgrade"
        }
    };

    let build_context = &u::root();
    let req_cmd = gen_req_cmd(dir);
    let image = find_image(&runtime);

    let f = format!(
        r#"
FROM {image} AS intermediate

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY pyproject.toml ./

COPY --from=shared . {build_context}/

RUN {req_cmd}

RUN mkdir -p /build/lib

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. pip install -vvv -r requirements.txt --target=/build/python --implementation cp --only-binary=:all: --upgrade

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
