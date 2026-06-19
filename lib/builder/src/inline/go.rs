use kit as u;

pub fn gen_dockerfile(dir: &str) {
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-provided.al2023:1.161

WORKDIR /build
COPY . .

ENV GOOS=linux
ENV CGO_ENABLED=0

RUN go build -tags lambda.norpc -o bootstrap
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
