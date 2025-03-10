use kit as u;

fn gen_dockerfile(dir: &str) {
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


fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    u::sh(&clean, dir);
    u::sh(&run, dir);
    let id = u::sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);

    u::sh(&format!("docker cp {}:/build/target/lambda/bootstrap/bootstrap bootstrap", id), dir);
    u::sh(&clean, dir);
    u::sh("rm -f Dockerfile wrapper", dir);
}

pub fn build(dir: &str) -> String {

    let no_docker = match std::env::var("TC_NO_DOCKER_BUILD") {
        Ok(_) => true,
        Err(_) => false
    };
    if no_docker {
        let cmds = vec![
            "rustup target add x86_64-unknown-linux-musl",
            "cargo build --release --target x86_64-unknown-linux-musl --target-dir build",
            "cp build/x86_64-unknown-linux-musl/release/bootstrap bootstrap",
        ];
        u::run_seq(cmds, dir);
    } else {
        gen_dockerfile(dir);
        let name = u::basedir(dir);
        u::runcmd_stream(&format!("docker build --no-cache  -t {} .", name), dir);
        copy_from_docker(dir);
        u::runcmd_stream("rm -rf build target Dockerfile", dir);
    }
    if !u::path_exists(dir, "bootstrap") {
        panic!("Building failed");
    }

    let size = u::path_size(dir, "bootstrap");

   println!("Built bootstrap ({})", u::file_size_human(size));
    let command = "zip -q -r lambda.zip bootstrap";
    u::sh(command, dir);
    format!("{}/lambda.zip", dir)
}
