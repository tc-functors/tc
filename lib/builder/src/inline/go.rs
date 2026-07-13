use compiler::Arch;
use kit as u;

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 0")
    }
}

pub fn gen_dockerfile(dir: &str, arch: &Arch, pre: &Vec<String>) {
    let pre = deps_str(pre.to_vec());
    let arch_env = match arch {
        Arch::Arm64 => "GOARCH=arm64",
        Arch::X8664 => "GOARCH=amd64",
    };

    let f = format!(
        r#"
FROM golang:1.25-alpine AS builder

ENV GOOS=linux
ENV {arch_env}
ENV CGO_ENABLED=0
ENV GIT_SSH_COMMAND="ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new"

RUN apk add --no-cache git openssh-client

WORKDIR /build
COPY . .

RUN --mount=type=ssh \
    {pre} && \
    go mod download && \
    go build -tags lambda.norpc -o bootstrap
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
