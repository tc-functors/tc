use kit::sh;
use kit as u;

fn extension_wrapper(name: &str) -> String {
    format!(
        r#"#!/bin/bash
set -euo pipefail

echo "{name}  launching extension"
exec "/opt/{name}/extension.py"

"#
    )
}

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
}

pub fn build(dir: &str, name: &str) -> String {
    sh("rm -rf *.zip build", dir);
    sh(&format!("mkdir -p build/{}", name), dir);
    sh(&format!("cp extension.py build/{}/", name), dir);

    u::mkdir("build/extensions");
    let wrapper_str = extension_wrapper(name);
    u::write_str(&format!("build/extensions/{}", name), &wrapper_str);
    sh("rm -f *.zip", dir);
    sh("cd build && zip -r -q ../extension.zip .", dir);
    sh("rm -rf build", dir);
    let size = size_of(dir, "extension.zip");
    println!("Size: {}", size);
    format!("{}/extension.zip", dir)
}
