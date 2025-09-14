use kit as u;

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
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

fn deps_str(deps: Vec<String>) -> String {
    if deps.len() >= 2 {
        deps.join(" && ")
    } else if deps.len() == 1 {
        deps.first().unwrap().to_string()
    } else {
        String::from("echo 0")
    }
}

pub fn gen_dockerfile_no_wrap(dir: &str, pre: &Vec<String>, post: &Vec<String>) {
    let pre = deps_str(pre.to_vec());
    let post = deps_str(post.to_vec());
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

RUN {pre}

RUN --mount=type=ssh --mount=type=cache,target=/.root/cache bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install && bundle lock && bundle install

RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/ && cp Gemfile /build/ruby/
RUN mkdir -p /build/ruby/vendor
RUN cp -r vendor/cache /build/ruby/vendor/cache
RUN rm -rf vendor ruby /build/ruby/lib/cache/
RUN --mount=type=ssh --mount=type=secret,id=aws,target=/root/.aws/credentials {post}
RUN --mount=type=cache,target=/.root/cache {extra_str}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn gen_dockerfile(dir: &str, pre: &Vec<String>, post: &Vec<String>) {
    let pre = deps_str(pre.to_vec());
    let post = deps_str(post.to_vec());
    let build_context = &top_level();
    let extra_str = u::vec_to_str(shared_objects());
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-ruby3.2:1.103.0-20231116224730 AS intermediate
WORKDIR {dir}

RUN mkdir -p -m 0600 ~/.ssh && ssh-keyscan github.com >> ~/.ssh/known_hosts
COPY Gemfile ./
COPY Gemfile.lock ./

COPY --from=shared . {build_context}/

RUN sed -i "/group/,/end:/d" Gemfile

RUN mkdir -p /build/ruby/lib /build/lib

RUN {pre}

RUN --mount=type=ssh --mount=type=cache,target=/.root/cache BUNDLE_WITHOUT="test:development" bundle config set --local without development test && bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install
RUN --mount=type=ssh BUNDLE_WITHOUT="test:development" bundle install --without development test

ENV BUNDLE_WITHOUT "test:development"
RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/Gemfile.lock && cp Gemfile /build/ruby/
RUN mkdir -p /build/ruby/vendor
RUN cp -r vendor/cache /build/ruby/vendor/cache
RUN rm -rf vendor ruby /build/ruby/lib/cache/
RUN --mount=type=ssh --mount=type=secret,id=aws,target=/root/.aws/credentials {post}
RUN --mount=type=cache,target=/.root/cache {extra_str}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

pub fn gen_dockerfile_unshared(dir: &str, pre: &Vec<String>, post: &Vec<String>) {
    let pre = deps_str(pre.to_vec());
    let post = deps_str(post.to_vec());
    let extra_str = u::vec_to_str(shared_objects());
    let f = format!(
        r#"
FROM public.ecr.aws/sam/build-ruby3.2:1.103.0-20231116224730 AS intermediate
WORKDIR {dir}

COPY Gemfile ./

RUN sed -i "/group/,/end:/d" Gemfile

RUN mkdir -p /build/ruby/lib /build/lib

RUN {pre}

ENV BUNDLE_WITHOUT "test:development"
RUN --mount=type=ssh --mount=type=cache,target=/.root/cache BUNDLE_WITHOUT="test:development" bundle config set --local without development test && bundle config set path vendor/bundle && bundle config set cache_all true && bundle cache --no-install && bundle lock && bundle install --without development test

RUN mkdir -p /build/ruby/gems
RUN mv vendor/bundle/ruby/3.2.0 /build/ruby/gems/3.2.0
RUN cp Gemfile.lock /build/ruby/ && cp Gemfile /build/ruby/
RUN mkdir -p /build/ruby/vendor
RUN cp -r vendor/cache /build/ruby/vendor/cache
RUN rm -rf vendor ruby /build/ruby/lib/cache/
RUN {post}
RUN --mount=type=cache,target=/.root/cache {extra_str}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
