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
