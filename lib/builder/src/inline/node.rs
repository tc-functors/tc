use kit as u;


pub fn gen_dockerfile(dir: &str) {
    let install_cmd = "NODE_ENV=production yarn install --production";
    let image = "node:22-alpine3.19";

    let token = match std::env::var("CODEARTIFACT_AUTH_TOKEN") {
        Ok(t) => t,
        Err(_) => String::from("")
    };

    let f = format!(
        r#"

FROM {image} AS intermediate

ARG AUTH_TOKEN {token}
ENV CODEARTIFACT_AUTH_TOKEN $AUTH_TOKEN
ENV NODE_ENV production

WORKDIR /build

RUN rm -rf /build/node_modules && mkdir -p /build
COPY . /build

RUN {install_cmd}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
