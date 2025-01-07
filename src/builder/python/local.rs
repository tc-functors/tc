use colored::Colorize;
use kit as u;

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
}

fn copy(dir: &str) {
    if u::path_exists(dir, "src") {
        u::sh("cp -r src/* build/python/", dir);
    }
    if u::path_exists(dir, "lib") {
        u::sh("cp -r lib/* build/python/", dir);
    }
}

fn zip(dir: &str, zipfile: &str) {
    if u::path_exists(dir, "build") {
        let cmd = format!("cd build && zip -q -9 -r ../{} . && cd -", zipfile);
        u::runcmd_quiet(&cmd, dir);
    }
}

pub fn build(dir: &str, name: &str) {
    let bar = kit::progress();

    let prefix = format!("Building {}", name.blue());
    bar.set_prefix(prefix);

    if u::path_exists(dir, "pyproject.toml") {
        bar.inc(10);
        u::sh("poetry config warnings.export false", dir);
        let cmd = "rm -f requirements.txt && poetry export -f requirements.txt --output requirements.txt --without-hashes --without dev";
        bar.inc(30);
        u::sh(cmd, dir);
        u::sh("poetry build", dir);
        bar.inc(50);
        let c = "poetry run pip install -r requirements.txt --platform manylinux2014_x86_64 --no-deps --upgrade --target build/python";
        u::sh(c, dir);
        bar.inc(60);
        if !u::path_exists(dir, "function.json") {
            copy(dir);
        }
        bar.inc(80);
        zip(dir, "deps.zip");

        bar.inc(100);
        let size = format!("({})", size_of(dir, "deps.zip").green());
        bar.set_message(size);
        bar.finish();
        u::sh("rm -f requirements.txt", dir);
    }
}
