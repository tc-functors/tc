use kit as u;


pub fn gen_dockerfile(dir: &str) {
    let install_cmd = "npm install";
    let image = "public.ecr.aws/sam/build-nodejs20.x:latest";

    let f = format!(
        r#"
FROM {image} AS intermediate
WORKDIR /build

RUN rm -rf /build/node_modules && mkdir -p /build
RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY package.json /build/

RUN {install_cmd}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
