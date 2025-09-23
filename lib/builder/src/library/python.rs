use kit as u;
use kit::sh;
use crate::layer;
use composer::LangRuntime;

pub fn build(dir: &str) -> String {
    u::run("rm -rf deps.zip build", &dir);
    let dirs = u::list_dir(dir);
    u::run("mkdir -p build/python/lib && mkdir -p build/lib", &dir);
    for d in dirs {
        if !d.ends_with("build") {
            let cmd = format!(
                "cp -r {}/src/* build/python/",
                &d
            );
            u::run(&cmd, &dir);
            let langr = LangRuntime::Python310;

            println!("Building {}", &d);

            layer::gen_dockerfile(&d, &langr);
            let (status, out, err) = layer::build_with_docker(&d);
            layer::copy_from_docker(&d);
            let cmd = format!("cp -rv . {}/build", dir);
            println!("cmd {}", &cmd);
            sh(&cmd, &format!("{}/build", &d));
            sh(&format!("rm -rf {}/build", &d), dir);
        }
    }
    u::run("zip -q -9 -r ../deps.zip .", &format!("{}/build", dir));
    let size = u::path_size(dir, "deps.zip");
    println!("Merged library ({})", u::file_size_human(size));
    format!("{}/deps.zip", dir)
}
