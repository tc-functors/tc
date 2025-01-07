use colored::Colorize;
use kit as u;

fn gen_dockerfile(dir: &str) {
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-provided.al2023:1.113.0-20240319235114 as intermediate

ENV JANET_TREE /build
ENV JANET_PATH /build/lib
ENV JANET_HEADERPATH /build/include
ENV JANET_SYSPATH /build/lib
ENV JANET_LIBPATH /build/lib
ENV JANET_BINPATH /build/bin
ENV JANET_TREE /build
ENV PREFIX /build

RUN dnf update --releasever=2023.0.20230210
RUN dnf update && dnf install git gcc curl-devel make -y

COPY . /app
RUN mkdir -p /build

RUN mkdir -p /tmp && cd /tmp && git clone https://github.com/janet-lang/janet.git -b v1.33.0 && cd janet && make && PREFIX=/build make install

RUN cd /tmp && git clone --depth=1 https://github.com/janet-lang/jpm.git && cd jpm && /build/bin/janet bootstrap.janet

RUN cd /app && /build/bin/jpm deps && /build/bin/jpm build

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    u::runcmd_quiet(&clean, dir);
    u::sh(&run, dir);
    let id = u::sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    if id.is_empty() {
        println!("{}: ", dir);
        u::sh("rm -f requirements.txt Dockerfile", dir);
        std::panic::set_hook(Box::new(|_| {
            println!("Build failed");
        }));
        panic!("build failed")
    }
    u::sh(&format!("docker cp {}:/build build", id), dir);
    u::sh(&clean, dir);
}

pub fn run_docker(dir: &str, trace: bool) {
    let root = &git::root();
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --ssh default={} -t {} --build-context shared={root} .",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --ssh default -t {} --build-context shared={root} .",
            u::basedir(dir)
        ),
    };
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
    println!("Building runtime: {} (janet)", u::basedir(dir).blue());
    gen_dockerfile(dir);
    run_docker(dir, trace);
    copy_from_docker(dir);
}
