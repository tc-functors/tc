use kit as u;


pub fn build(dir: &str, _name: &str) -> String {
    u::sh("rm -f deps.zip build", &dir);
    let dirs = u::list_dir(dir);
    u::runcmd_quiet("mkdir -p build/ruby/lib", &dir);
    for d in dirs {
        if !d.ends_with("build") {
            let cmd = format!("cp -r {}/lib/* build/lib/", &d);
            u::runcmd_stream(&cmd, &dir);
        }
    }
    u::runcmd_quiet("cd build && zip -q -9 -r ../deps.zip .", &dir);
    let size = u::path_size(dir, "deps.zip");
    println!("Merged layer ({})", u::file_size_human(size));
    format!("{}/deps.zip", dir)
}
