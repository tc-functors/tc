use kit as u;

pub fn gen_dockerfile(dir: &str) {
    let install_cmd = "yarn install --no-lockfile --production";
    let image = "node:22-alpine3.19";

    let token = match std::env::var("CODEARTIFACT_AUTH_TOKEN") {
        Ok(t) => t,
        Err(_) => String::from(""),
    };

    let extra_cmd = if u::path_exists(dir, "build.js") {
        "node build.js"
    } else {
        "echo 1"
    };

    let f = format!(
        r#"

FROM {image} AS intermediate

ARG AUTH_TOKEN {token}
ENV CODEARTIFACT_AUTH_TOKEN $AUTH_TOKEN

WORKDIR /build

RUN rm -rf /build/node_modules && mkdir -p /build
COPY . /build

RUN {install_cmd}

RUN {extra_cmd}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
