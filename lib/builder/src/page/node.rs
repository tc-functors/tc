use kit as u;

pub fn gen_dockerfile(dir: &str, command: &str) {
    let image = "node:22-alpine3.19";

    let f = format!(
        r#"

FROM {image} AS intermediate

WORKDIR /build

RUN rm -rf /build/node_modules && mkdir -p /build
COPY . /build

RUN {command}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
