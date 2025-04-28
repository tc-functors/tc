use kit as u;
use kit::sh;

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
}

fn gen_dockerignore(dir: &str) {
    let f = format!(r#"
**/node_modules/
**/dist
**/logs
**/target
**/vendor
**/build
.git
npm-debug.log
.coverage
.coverage.*
.env
*.zip
"#);
    let file = format!("{}/.dockerignore", dir);
    u::write_str(&file, &f);
}

fn shared_objects() -> Vec<&'static str> {
    vec![
        "cp /usr/lib64/libnghttp2.so.14.20.0 /build/ruby/lib/libnghttp2.so.14",
        "&& cp /usr/lib64/libcurl.so.4.8.0 /build/ruby/lib/libcurl.so.4",
        "&& cp /usr/lib64/libpsl* /build/ruby/lib/",
        "&& cp /usr/lib64/libidn2.so.0.3.7 /build/ruby/lib/libidn2.so.0",
        "&& cp /usr/lib64/liblber-2.4.so.2.10.7 /build/ruby/lib/liblber-2.4.so.2",
        "&& cp /usr/lib64/libldap-2.4.so.2.10.7 /build/ruby/lib/libldap-2.4.so.2",
        "&& cp /usr/lib64/libnss3.so /build/ruby/lib/libnss3.so",
        "&& cp /usr/lib64/libnssutil3.so /build/ruby/lib/libnssutil3.so",
        "&& cp /usr/lib64/libsmime3.so /build/ruby/lib/libsmime3.so",
        "&& cp /usr/lib64/libssl3.so /build/ruby/lib/libssl3.so",
        "&& cp /usr/lib64/libunistring.so.0.1.2 /build/ruby/lib/libunistring.so.0",
        "&& cp /usr/lib64/libsasl2.so.3.0.0 /build/ruby/lib/libsasl2.so.3",
        "&& cp /usr/lib64/libssh2.so.1.0.1 /build/ruby/lib/libssh2.so.1",
        "&& cp /usr/lib64/libffi.so.6 /build/ruby/lib/libffi.so.6",
    ]
}

fn gen_dockerfile(dir: &str) {
    let build_context = &top_level();
    let extra_str = u::vec_to_str(shared_objects());
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-ruby3.2:1.103.0-20231116224730 AS intermediate
WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY Gemfile ./

COPY --from=shared . {build_context}/

RUN mkdir -p /build/ruby/lib /build/lib

RUN yum update -yy

RUN yum -y install libffi.x86_64 libpsl-devel

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install && bundle lock && bundle install

RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/ && cp Gemfile /build/ruby/
RUN mkdir -p /build/ruby/vendor
RUN cp -r vendor/cache /build/ruby/vendor/cache
RUN rm -rf vendor ruby /build/ruby/lib/cache/
RUN {extra_str}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn build_with_docker(dir: &str) {
    let root = &top_level();
    let cmd_str = match std::env::var("DOCKER_SSH") {
        Ok(e) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default={} -t {} --build-context shared={root} .",
            &e,
            u::basedir(dir)
        ),
        Err(_) => format!(
            "docker buildx build --platform=linux/amd64 --ssh default  -t {} --build-context shared={root} .",
            u::basedir(dir)
        ),
    };
    let status = u::runp(&cmd_str, dir);
    if !status {
        sh("rm -f Dockerfile wrapper", dir);
        panic!("Failed to build");
    }
}

fn copy_from_docker(dir: &str) {
    let temp_cont = &format!("tmp-{}", u::basedir(dir));
    let clean = &format!("docker rm -f {}", &temp_cont);

    let run = format!("docker run -d --name {} {}", &temp_cont, u::basedir(dir));
    sh(&clean, dir);
    sh(&run, dir);
    let id = u::sh(&format!("docker ps -aqf \"name={}\"", temp_cont), dir);
    tracing::debug!("Container id: {}", &id);

    sh(&format!("docker cp {}:/build build", id), dir);
    sh(&clean, dir);
    sh("rm -f Dockerfile wrapper", dir);
}

fn build_docker(dir: &str) {
    gen_dockerfile(dir);
    gen_dockerignore(dir);
    build_with_docker(dir);
    copy_from_docker(dir);
    sh("rm -f Dockerfile wrapper .dockerignore", dir);
    let cmd = "cd build/ruby && find . -type d -name \".git\" | xargs rm -rf && rm -rf gems/3.2.0/cache/bundler/git && zip -q -9 --exclude=\"**/.git/**\" -r ../../lambda.zip . && cd -";
    sh(&cmd, dir);
}

pub fn build(dir: &str, _name: &str, given_command: &str) -> String {
    sh("rm -f lambda.zip deps.zip build", dir);
    build_docker(dir);
    sh(given_command, dir);
    sh("rm -rf build build.json", dir);
    format!("{}/lambda.zip", dir)
}
