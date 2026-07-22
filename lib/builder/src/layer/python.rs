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
        LangRuntime::Python314 => String::from("public.ecr.aws/sam/build-python3.14:latest"),
        _ => todo!(),
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

fn make_install_command(dir: &str, package_manager: &str) -> String {
    match package_manager  {
        "uv" =>  {
            if u::path_exists(dir, "pyproject.toml") {
                format!("uv sync --no-dev && uv pip install -r pyproject.toml --target=/build/python")
            } else if u::path_exists(dir, "requirements.txt") {
                format!("uv pip install -r requirements.txt --target=/build/python")
            } else {
                format!("echo 0")
            }
        },
        _ => panic!("Please use uv as package manager for layers")
    }
}

pub fn gen_dockerfile(dir: &str, runtime: &LangRuntime, package_manager: &str) {
    let _extra_str = u::vec_to_str(shared_objects());

    let build_context = &u::root();
    let install_cmd = make_install_command(dir, package_manager);
    let copy_cmd = make_copy_cmd(dir);
    let image = find_image(&runtime);

    let f = format!(
        r#"
FROM {image} AS intermediate

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

{copy_cmd}

COPY --from=shared . {build_context}/

RUN mkdir -p /build/lib

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. {install_cmd}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
