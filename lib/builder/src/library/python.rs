use crate::layer;
use compiler::spec::LangRuntime;
use composer::Build;
use kit as u;
use kit::sh;

pub fn build(
    dir: &str,
    langr: &LangRuntime,
    dirs: &Vec<String>,
    include_deps: bool,
    bspec: &Build
) -> String {
    let post = &bspec.post;
    u::run("rm -rf deps.zip build", &dir);
    u::run("mkdir -p build/python/lib && mkdir -p build/lib", &dir);

    for d in dirs {
        println!("Building {}", &d);
        let cmd = if u::path_exists(&d, "src") {
            format!("cp -r {}/src/* build/python/", &d)
        } else {
            format!("cp -r {}/* build/python/", &d)
        };
        u::run(&cmd, &dir);

        println!("include_deps {}", include_deps);
        if include_deps {
            layer::gen_dockerfile(&d, &langr, &bspec.package_manager);
            let (_status, _out, _err) = layer::build_with_docker(&d);
            layer::copy_from_docker(&d);
            let cmd = format!("cp -rv . {}/build", dir);
            println!("cmd {}", &cmd);
            sh(&cmd, &format!("{}/build", &d));
            sh(&format!("rm -rf {}/build", &d), dir);
        }
    }
    for cmd in post {
        u::runcmd_stream(&cmd, dir);
    }

    u::run("zip -q -9 -r ../deps.zip .", &format!("{}/build", dir));
    let size = u::path_size(dir, "deps.zip");
    println!("Merged library ({})", u::file_size_human(size));
    u::run("rm -rf build", &dir);
    format!("{}/deps.zip", dir)
}
