use kit as u;
use kit::sh;

pub fn build(dir: &str) -> String {
    let cmd = vec![
        "docker run --rm",
        "-v `pwd`:/code -w /code",
        "-v ${HOME}/.cargo/registry:/cargo/registry",
        "-v ${HOME}/.cargo/git:/cargo/git",
        "-u $(id -u):$(id -g)",
        "rustserverless/lambda-rust",
    ];
    u::runv(dir, cmd);
    let name = u::sh(
        "cargo metadata --no-deps --format-version 1 | jq -r '.packages[].targets[] | select( .kind | map(. == \"bin\") | any ) | .name'",
        dir,
    );
    let cmd = format!(
        "mkdir -p extensions && cp target/lambda/release/{} extensions/",
        name
    );
    u::runcmd_stream(&cmd, dir);
    sh("rm -rf build target", dir);
    u::runcmd_stream("zip -q -9 -r extension.zip extensions", &u::pwd());
    u::runcmd_stream("rm -rf extensions", &u::pwd());
    let size = u::file_size("extension.zip");
    println!("Built extension ({})", u::file_size_human(size));

    format!("{}/extension.zip", dir)
}
