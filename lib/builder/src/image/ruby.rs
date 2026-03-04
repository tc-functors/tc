use compiler::spec::LangRuntime;
use kit as u;

fn find_build_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Ruby32 => "ruby3.2:1.103.0-20231116224730",
        _ => todo!(),
    };
    format!("public.ecr.aws/sam/build-{}", &tag)
}

fn shared_objects() -> Vec<&'static str> {
    vec![
        "cp /usr/lib64/libnghttp2.so.14.20.0 /build/ruby/lib/libnghttp2.so.14",
        "&& cp /usr/lib64/libcurl.so.4.8.0 /build/ruby/lib/libcurl.so.4",
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
    ]
}

fn find_runtime_image(runtime: &LangRuntime) -> String {
    let tag = match runtime {
        LangRuntime::Ruby32 => "ruby:3.2",
        _ => todo!(),
    };
    format!("public.ecr.aws/lambda/{}", &tag)
}

fn deps_str(deps: &Vec<String>) -> String {
    let s = if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 0")
    };
    s.replace("AWS_PROFILE=cicd", "")
}

pub fn gen_base_dockerfile(
    dir: &str,
    runtime: &LangRuntime,
    pre: &Vec<String>,
    post: &Vec<String>,
) {
    let pre_commands = deps_str(pre);

    let post_commands = deps_str(post);

    let build_image = find_build_image(runtime);
    let runtime_image = find_runtime_image(runtime);

    let build_context = &u::root();
    let extra_str = u::vec_to_str(shared_objects());

    let f = format!(
        r#"
FROM {build_image} AS build-image

WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts

COPY Gemfile ./

COPY --from=shared . {build_context}/

RUN mkdir -p /build/ruby/lib /build/lib

RUN {pre_commands}

RUN --mount=type=ssh --mount=target=shared,type=bind,source=. --mount=type=cache,target=/.root/cache bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install && bundle lock && bundle install

RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/ && cp Gemfile /build/ruby/
RUN mkdir -p /build/ruby/vendor
RUN cp -r vendor/cache /build/ruby/vendor/cache
RUN rm -rf vendor ruby /build/ruby/lib/cache/
RUN --mount=type=ssh --mount=type=secret,id=aws,target=/root/.aws/credentials {post_commands}
RUN --mount=type=cache,target=/.root/cache {extra_str}

FROM {runtime_image} AS runtime

COPY --from=build-image /build/ruby /opt/ruby

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn gen_code_dockerfile(dir: &str, base_image: &str) {
    let f = format!(
        r#"
FROM {base_image}

COPY . /var/task

CMD [ "handler.handler" ]
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
