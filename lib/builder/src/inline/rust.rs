use kit as u;

pub fn gen_dockerfile(dir: &str) {
    let f = format!(
        r#"
FROM ghcr.io/cargo-lambda/cargo-lambda:latest

WORKDIR /build
COPY . .

RUN cargo lambda build --release
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
