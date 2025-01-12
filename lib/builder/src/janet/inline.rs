use colored::Colorize;
use kit as u;

fn gen_dockerfile(dir: &str) {
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-provided.al2023:1.113.0-20240319235114 as intermediate

ENV JANET_TREE /janet
ENV JANET_PATH /janet/lib
ENV JANET_HEADERPATH /janet/include
ENV JANET_SYSPATH /janet/lib
ENV JANET_LIBPATH /janet/lib
ENV JANET_BINPATH /janet/bin
ENV JANET_TREE /janet
ENV PREFIX /janet

RUN dnf update --releasever=2023.0.20230210
RUN dnf update && dnf install git gcc curl-devel make -y

COPY . /app

RUN mkdir -p /janet

RUN mkdir -p /tmp && cd /tmp && git clone https://github.com/janet-lang/janet.git -b v1.33.0 && cd janet && make && make install

RUN git clone --depth=1 https://github.com/janet-lang/jpm.git && cd jpm && /janet/bin/janet bootstrap.janet

RUN cd /app && /janet/bin/jpm deps && /janet/bin/jpm build

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn copy_from_docker(dir: &str, trace: bool) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    u::sh(&clean, dir);
    u::sh(&run, dir);
    let id = u::sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    if trace {
        println!("Container id: {}", &id);
    }
    u::sh(
        &format!("docker cp {}:/app/build/bootstrap bootstrap", id),
        dir,
    );
    u::sh(&clean, dir);
    u::sh("rm -f Dockerfile", dir);
}

fn run_docker(dir: &str, trace: bool) {
    let cmd_str = format!("docker build -t {} .", u::basedir(dir));
    let ret = u::runp(&cmd_str, dir, trace);
    if !ret {
        u::sh("rm -rf Dockerfile build", dir);
        std::panic::set_hook(Box::new(|_| {
            println!("Build failed");
        }));
        panic!("Build failed")
    }
}

pub fn build(dir: &str, trace: bool) {
    println!("Building {} (janet)", u::basedir(dir).blue());
    gen_dockerfile(dir);
    run_docker(dir, trace);
    copy_from_docker(dir, trace);
    let size = u::path_size(dir, "bootstrap");
    println!("Built bootstrap ({})", u::file_size_human(size));
}
