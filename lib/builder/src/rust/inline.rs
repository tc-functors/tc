use colored::Colorize;
use kit as u;

pub fn build(dir: &str) -> String {

    println!("Building {} (rust)", u::basedir(dir).blue());
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
        let cmd = vec![
            "docker run --rm",
            "-v `pwd`:/code -w /code",
            "-v ${HOME}/.cargo/registry:/cargo/registry",
            "-v ${HOME}/.cargo/git:/cargo/git",
            "-u $(id -u):$(id -g)",
            "rustserverless/lambda-rust",
        ];
        u::runv(dir, cmd);
        u::runcmd_stream("cp target/*/release/bootstrap bootstrap", dir);
        u::runcmd_stream("rm -rf build target", dir);
    }
    let size = u::path_size(dir, "bootstrap");
    println!("Built bootstrap ({})", u::file_size_human(size));
    let command = "zip -q -r lambda.zip bootstrap";
    u::sh(command, dir);
    format!("{}/lambda.zip", dir)
}
