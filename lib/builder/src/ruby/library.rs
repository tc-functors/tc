use kit as u;

pub fn build(dir: &str) -> String {
    u::run("rm -rf deps.zip build", &dir);
    let dirs = u::list_dir(dir);
    u::run("mkdir -p build/ruby/lib && mkdir -p build/lib", &dir);
    for d in dirs {
        if !d.ends_with("build") {
            let cmd = format!(
                "cp -r {}/lib/* build/ruby/lib/ && cp -r {}/lib/* build/lib/",
                &d, &d
            );
            u::run(&cmd, &dir);
        }
    }
    u::run("cd build && zip -q -9 -r ../deps.zip .", &dir);
    let size = u::path_size(dir, "deps.zip");
    println!("Merged library ({})", u::file_size_human(size));
    format!("{}/deps.zip", dir)
}

pub fn _merge_dirs(dirs: Vec<String>) {
    let cwd = u::pwd();
    let zipfile = format!("{}/deps.zip", &cwd);
    let build_dir = format!("{}/build", &cwd);
    u::sh(&format!("mkdir -p {}/ruby/lib", &build_dir), &cwd);
    for dir in dirs {
        if u::path_exists(&dir, "lib") && !&dir.ends_with("layer") && !&dir.ends_with("build") {
            u::sh(&format!("cp -r lib/* {}/ruby/lib/", &build_dir), &dir);
            u::sh(&format!("cp -r lib/* {}/lib/", &build_dir), &dir);
        }
    }
    let cmd = format!("zip -9 -q -r {} .", zipfile);
    u::sh(&cmd, &build_dir);
    let size = u::path_size(&cwd, "deps.zip");
    u::sh(&format!("rm -rf {}", &build_dir), &cwd);
    println!("Merged deps ({})", u::file_size_human(size));
    println!("To publish, run `tc publish --name NAME`")
}
