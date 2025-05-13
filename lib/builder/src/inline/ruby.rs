use kit as u;

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
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


pub fn gen_dockerfile(dir: &str) {
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
