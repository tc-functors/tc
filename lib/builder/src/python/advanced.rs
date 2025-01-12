use kit as u;

fn run(dir: &str, cmd: Vec<&str>, trace: bool) {
    let cmd_str = u::vec_to_str(cmd);
    if trace {
        u::runcmd_stream(&cmd_str, dir);
    } else {
        u::runcmd_quiet(&cmd_str, dir);
    }
}

// FIXME: use ldd
fn shared_objects(lang: &str) -> Vec<&'static str> {
    if lang != "python3.7" {
        vec![
            "cp -r /usr/lib64/libnghttp2.so.14.20.0 /build/lib/libnghttp2.so.14",
            "&& cp /usr/lib64/libcurl.so.4.8.0 /build/lib/libcurl.so.4",
            "&& cp /usr/lib64/libidn2.so.0.3.7 /build/lib/libidn2.so.0",
            "&& cp /usr/lib64/liblber-2.4.so.2.10.7 /build/lib/liblber-2.4.so.2",
            "&& cp /usr/lib64/libldap-2.4.so.2.10.7 /build/lib/libldap-2.4.so.2",
            "&& cp /usr/lib64/libnss3.so /build/lib/libnss3.so",
            "&& cp /usr/lib64/libsmime3.so /build/lib/libsmime3.so",
            "&& cp /usr/lib64/libssl3.so /build/lib/libssl3.so",
            "&& cp /usr/lib64/libunistring.so.0.1.2 /build/lib/libunistring.so.0",
            "&& cp /usr/lib64/libsasl2.so.3.0.0 /build/lib/libsasl2.so.3",
            "&& cp /usr/lib64/libssh2.so.1.0.1 /build/lib/libssh2.so.1",
            "&& cp --preserve=links /usr/lib64/libSM.so.6* /build/lib/",
            "&& cp --preserve=links /usr/lib64/libXrender.so.1* /build/lib/",
            "&& cp --preserve=links /usr/lib64/libXext.so.6* /build/lib/",
        ]
    } else {
        vec!["cp /usr/lib64/libssl3.so /build/lib/libssl3.so"]
    }
}

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 1")
    }
}

pub fn gen_dockerfile(dir: &str, lang: &str, deps_pre: Vec<String>, deps_post: Vec<String>) {
    let extra_str = u::vec_to_str(shared_objects(lang));
    let extra_deps_pre = deps_str(deps_pre);
    let extra_deps_post = deps_str(deps_post);

    let pip_cmd = match std::env::var("TC_FORCE_BUILD") {
        Ok(_) => "pip install -r requirements.txt --target=/build/python --upgrade",
        Err(_) => "pip install -r requirements.txt --platform manylinux2014_x86_64 --target=/build/python --implementation cp --only-binary=:all: --upgrade"
    };

    if lang == "python3.12" {
        let f = format!(
            r#"
FROM public.ecr.aws/sam/build-{lang}:latest as intermediate

ENV AWS_PROFILE=cicd

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY requirements.txt ./

ENV PATH $HOME/.cargo/bin:$PATH

RUN mkdir -p /build/lib

RUN dnf update -yy

RUN dnf -y install libXext libSM libXrender

RUN --mount=type=ssh pip install -vvv -r requirements.txt --target=/build/python --implementation cp --only-binary=:all: --upgrade

"#
        );
        let dockerfile = format!("{}/Dockerfile", dir);
        u::write_str(&dockerfile, &f);
    } else {
        let f = format!(
            r#"
FROM public.ecr.aws/sam/build-{lang}:latest as intermediate

ENV AWS_PROFILE=cicd

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY requirements.txt ./

ENV PATH $HOME/.cargo/bin:$PATH

RUN mkdir -p /build/lib

RUN yum update -yy

RUN yum -y install libXext libSM libXrender

RUN {extra_deps_pre}

RUN {extra_str}

RUN --mount=type=ssh {pip_cmd}

RUN --mount=type=secret,id=aws,target=/root/.aws/credentials {extra_deps_post}

"#
        );
        let dockerfile = format!("{}/Dockerfile", dir);
        u::write_str(&dockerfile, &f);
    }
}

pub fn gen_requirements_txt(dir: &str, lang: &str, trace: bool) {
    let img = match lang {
        "python3.12" => "thehale/python-poetry:1.8.3-py3.12-slim",
        "python3.11" => "sunpeek/poetry:py3.11-slim",
        "python3.10" => "sunpeek/poetry:py3.10-slim",
        "python3.9" => "sunpeek/poetry:py3.9-slim",
        "python3.7" => "thehale/python-poetry:1.4.2-py3.7-slim",
        _ => "sunpeek/poetry:py3.10-slim",
    };

    if lang != "python3.7" {
        let cmd = vec![
            "docker run --rm --platform=linux/amd64",
            "-u $(id -u):$(id -g)",
            "-v `pwd`:`pwd`",
            "-w `pwd`",
            &img,
            "sh -c \"rm -f requirements.txt && poetry export --without-hashes --format=requirements.txt --without dev > requirements.txt\"",
        ];
        run(dir, cmd, trace);
    }
}

pub fn build(dir: &str, trace: bool) {
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!("docker build --no-cache  --ssh default={} --secret id=aws,src=$HOME/.aws/credentials . -t {}",
                         &e, u::basedir(dir)),
        Err(_) => format!("docker build --no-cache  --ssh default --secret id=aws,src=$HOME/.aws/credentials . -t {}",
                          u::basedir(dir))
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
