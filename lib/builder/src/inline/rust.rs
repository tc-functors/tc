use kit as u;

pub fn gen_dockerfile(dir: &str) {
    let f = format!(
        r#"
FROM ghcr.io/cargo-lambda/cargo-lambda:latest

WORKDIR /build
COPY . .

ENV RUST_TARGET_DIR=/root/.cargo/target

RUN  --mount=type=cache,target=/root/.cargo/target cargo lambda build --release
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
