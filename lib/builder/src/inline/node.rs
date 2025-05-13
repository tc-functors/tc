use colored::Colorize;
use kit as u;

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
}

fn zip(dir: &str, zipfile: &str) {
    if u::path_exists(dir, "build") {
        let cmd = format!("cd build && zip -q -9 -r ../{} . && cd -", zipfile);
        u::runcmd_quiet(&cmd, dir);
    }
}

pub fn build(dir: &str) -> String {
    u::sh("rm -rf node_modules lib/nodejs/node_modules", dir);
    u::sh("npm install --omit=dev", dir);
    u::sh("mv node_modules lib/nodejs", dir);
    zip(dir, "deps.zip");
    format!("{}/lambda.zip", dir)
}
