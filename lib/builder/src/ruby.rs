use colored::Colorize;
use kit as u;

fn gen_wrapper(dir: &str) {
    let f = format!(
        r#"
#!/usr/bin/env sh

export BUNDLE_WITHOUT='test:development'
BUNDLE_GEMFILE=/opt/ruby/Gemfile bundle exec $@
"#
    );
    let file = format!("{}/wrapper", dir);
    u::write_str(&file, &f);
}

fn shared_objects() -> Vec<&'static str> {
    vec![
        "cp /usr/lib64/libnghttp2.so.14.20.0 /build/lib/libnghttp2.so.14",
        "&& cp /usr/lib64/libcurl.so.4.8.0 /build/lib/libcurl.so.4",
        "&& cp /usr/lib64/libidn2.so.0.3.7 /build/lib/libidn2.so.0",
        "&& cp /usr/lib64/liblber-2.4.so.2.10.7 /build/lib/liblber-2.4.so.2",
        "&& cp /usr/lib64/libldap-2.4.so.2.10.7 /build/lib/libldap-2.4.so.2",
        "&& cp /usr/lib64/libnss3.so /build/lib/libnss3.so",
        "&& cp /usr/lib64/libnssutil3.so /build/lib/libnssutil3.so",
        "&& cp /usr/lib64/libsmime3.so /build/lib/libsmime3.so",
        "&& cp /usr/lib64/libssl3.so /build/lib/libssl3.so",
        "&& cp /usr/lib64/libunistring.so.0.1.2 /build/lib/libunistring.so.0",
        "&& cp /usr/lib64/libsasl2.so.3.0.0 /build/lib/libsasl2.so.3",
        "&& cp /usr/lib64/libssh2.so.1.0.1 /build/lib/libssh2.so.1",
    ]
}

fn gen_dockerfile(dir: &str) {
    let build_context = &top_level();
    let extra_str = u::vec_to_str(shared_objects());
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-ruby3.2:1.103.0-20231116224730 as intermediate
WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY Gemfile ./
COPY wrapper ./

COPY --from=shared . {build_context}/

RUN sed -i "/group/,/end:/d" Gemfile

RUN mkdir -p /build/ruby/lib /build/lib

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. BUNDLE_WITHOUT="test:development" bundle config set --local without development test && bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install

ENV BUNDLE_WITHOUT "test:development"
RUN --mount=type=ssh bundle lock && bundle install
RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/ && cp wrapper /build/ruby/ && cp Gemfile /build/ruby/
RUN find vendor/cache/ -maxdepth 1 -type d | xargs -I {{}} cp -r {{}} /build/ruby/lib/
RUN rm -rf vendor ruby /build/ruby/lib/cache/
RUN {extra_str}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn copy(dir: &str) {
    if u::path_exists(dir, "src") {
        u::sh("cp -r src/* build/ruby/", dir);
    }
    if u::path_exists(dir, "lib") {
        u::sh(
            "mkdir -p build/ruby/lib && cp -r lib/* build/ruby/lib/",
            dir,
        );
        u::runcmd_quiet("mkdir -p build/lib && cp -r lib/* build/lib/", dir);
    }
    let basedir = u::snake_case(&u::basename(dir));
    if u::path_exists(dir, &basedir) {
        let cp_cmd = format!("mkdir -p build/ruby/ && cp -r {} build/ruby/", &basedir);
        u::sh(&cp_cmd, dir);
        u::sh("cp *.rb build/ruby/", dir);
    }
}

pub fn zip(dir: &str, zipfile: &str) {
    if u::path_exists(dir, "build") {
        let cmd = format!("cd build && find . -type d -name \".git\" | xargs rm -rf && rm -rf ruby/gems/3.2.0/cache/bundler/git && zip -q -9 --exclude=\"**/.git/**\" -r ../{} . && cd -", zipfile);
        u::runcmd_quiet(&cmd, dir);
    }
}

fn size_of(dir: &str, zipfile: &str) -> String {
    let size = u::path_size(dir, zipfile);
    u::file_size_human(size)
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

    u::sh(&format!("docker cp {}:/build build", id), dir);
    u::sh(&clean, dir);
    u::sh("rm -f Dockerfile wrapper", dir);
}

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
}

fn build_with_docker(dir: &str, trace: bool) {
    let root = &top_level();
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --ssh default={} -t {} --build-context shared={root} .",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --ssh default  -t {} --build-context shared={root} .",
            u::basedir(dir)
        ),
    };
    let status = u::runp(&cmd_str, dir, trace);
    if !status {
        u::sh("rm -f Dockerfile wrapper", dir);
        panic!("Failed to build");
    }
}

pub fn build_deps(dir: &str, name: &str, _no_docker: bool, trace: bool) {
    u::sh("rm -f deps.zip", dir);
    let bar = kit::progress();
    let prefix = format!("Building   {}", name.blue());
    bar.set_prefix(prefix);
    bar.inc(10);
    gen_wrapper(dir);
    gen_dockerfile(dir);
    bar.inc(20);
    build_with_docker(dir, trace);
    bar.inc(50);
    copy_from_docker(dir, trace);
    bar.inc(70);
    if !u::path_exists(dir, "function.json") {
        copy(dir);
    }
    bar.inc(80);
    zip(dir, "deps.zip");
    bar.inc(100);
    u::runcmd_quiet("rm -rf vendor && rm -rf bundler", dir);
    let size = format!("({})", size_of(dir, "deps.zip").green());
    bar.set_message(size);
    bar.finish();
}

pub fn pack(dir: &str, command: &str) {
    u::sh("rm -f lambda.zip", dir);
    match command {
        "inline-deps" => {
            build_with_docker(dir, false);
            copy(dir);
            let cmd = "cd build/ruby && zip -q -9 -r ../../lambda.zip . && cd -";
            u::runcmd_quiet(&cmd, dir);
        }
        _ => {
            let c = format!(r"{}", command);
            u::sh(&c, dir);
            u::runcmd_quiet("zip -q -9 lambda.zip", dir);
        }
    }
}

pub fn clean(dir: &str) {
    u::sh("rm -f lambda.zip", dir);
}
