use kit as u;
use kit::sh;

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
}

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

fn gen_dockerfile(dir: &str) {
    let build_context = &top_level();
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

RUN BUNDLE_WITHOUT="test:development" bundle config set --local without development test && bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install

ENV BUNDLE_WITHOUT "test:development"
RUN --mount=type=ssh bundle lock && bundle install
RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/ && cp wrapper /build/ruby/ && cp Gemfile /build/ruby/
RUN find vendor/cache/ -maxdepth 1 -type d | xargs -I {{}} cp -r {{}} /build/ruby/lib/
RUN rm -rf vendor ruby /build/ruby/lib/cache/
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn build_with_docker(dir: &str) {
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

fn build_local(dir: &str) {
    let cmd = "BUNDLE_WITHOUT='test:development' bundle config set --local without development test && bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install";
    u::sh(&cmd, dir);
    sh("bundle lock && bundle install", dir);
    sh("mkdir -p build/ruby/gems", dir);
    sh("mv vendor/bundle/ruby/3.2.0 build/ruby/gems/3.2.0", dir);
    let cmd = "cd build/ruby && zip -q -9 -r ../../lambda.zip . && cd -";
    sh(cmd, dir);
    sh("rm -rf vendor build", dir);
}

fn build_docker(dir: &str) {
    gen_wrapper(dir);
    gen_dockerfile(dir);
    build_with_docker(dir);
    copy_from_docker(dir);
    sh("rm -f Dockerfile wrapper", dir);
    let cmd = "cd build/ruby && zip -q -9 -r ../../lambda.zip . && cd -";
    sh(&cmd, dir);
}

pub fn build(dir: &str, _name: &str, given_command: &str) -> String {
    sh("rm -f lambda.zip deps.zip build", dir);
    match std::env::var("TC_NO_DOCKER") {
        Ok(_) => build_local(dir),
        Err(_) => build_docker(dir)
    }
    sh(given_command, dir);
    sh("rm -rf build build.json", dir);
    format!("{}/lambda.zip", dir)
}
