mod inline;
mod runtime;

use kit as u;

pub fn clean(dir: &str) {
    u::runcmd_quiet("rm -rf deps.zip build target bootstrap", dir);
}

pub fn build_runtime(dir: &str, trace: bool) {
    runtime::build(dir, trace);
}

pub fn build_deps(dir: &str, trace: bool) {
    inline::build(dir, trace);
}

fn gen_bootstrap(dir: &str) {
    let content = format!(
        r#"
#!/bin/sh

set -euo pipefail
/opt/bin/janet $@
"#
    );
    let file = format!("{}/bootstrap", dir);
    u::write_str(&file, &content);
}

pub fn pack(dir: &str) {
    if !u::path_exists(dir, "runtime.janet") {
        gen_bootstrap(dir);
    }
    let command = "zip -q -r lambda.zip bootstrap";
    u::sh(command, dir);
}
