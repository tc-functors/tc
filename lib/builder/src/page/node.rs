use kit as u;
use kit::*;

fn find_copy_command(dir: &str, ct: &Option<String>) -> String {
    if let Some(p) = ct {
        let path = format!("{}/{}", dir, &p);
        let tmp = format!("{}_tmp", &p);
        if u::file_exists(&path) && u::file_exists(&tmp) {
            format!("cp {} {}", &tmp, &p)
        } else {
            s!("echo 1")
        }
    } else {
        s!("echo 1")
    }
}

pub fn gen_dockerfile(dir: &str, command: &str, config_template: &Option<String>) {
    let image = "node:22-alpine3.19";

    let copy_config_cmd = find_copy_command(dir, config_template);

    let f = format!(
        r#"

FROM {image} AS intermediate

WORKDIR /build

RUN rm -rf /build/node_modules && mkdir -p /build
COPY . /build

RUN {copy_config_cmd}

RUN cat .env

RUN {command}

"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}
