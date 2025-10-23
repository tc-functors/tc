use kit as u;
use kit::*;

fn find_copy_command(_dir: &str, ct: &Option<String>) -> String {
    if let Some(p) = ct {
        let tmp = format!("{}_tmp", &p);
        format!("cp {} {}", &tmp, &p)
    } else {
        s!("echo 1")
    }
}

pub fn gen_dockerfile(dir: &str, command: &str, config_template: &Option<String>) {
    let image = "node:22-alpine3.19";

    let copy_config_cmd = find_copy_command(dir, config_template);

    let token = match std::env::var("CODEARTIFACT_AUTH_TOKEN") {
        Ok(t) => t,
        Err(_) => String::from(""),
    };


    let f = format!(
        r#"

FROM {image} AS intermediate

ARG AUTH_TOKEN {token}
ENV CODEARTIFACT_AUTH_TOKEN $AUTH_TOKEN

WORKDIR /build

RUN rm -rf /build/node_modules && mkdir -p /build
COPY . /build

RUN {copy_config_cmd}

RUN {command}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
