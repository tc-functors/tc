use crate::types::BuildStatus;
use composer::{Build, Runtime};

use compiler::spec::{
    Lang,
};

use provider::Auth;
use provider::aws::microvm::MicroVmImage;
use provider::aws::microvm;
use provider::aws::s3;

use kit as u;

mod python;

fn make_cmd(handler: &str) -> String {
    let parts = handler.split(" ").collect::<Vec<&str>>();
    format!("{:?}", &parts)
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

fn gen_generic_dockerfile(dir: &str, handler: &str, port: &i32, pre: &Vec<String>) {
    let cmd = make_cmd(handler);
    let pre = deps_str(pre.to_vec());
    let f = format!(
        r#"
FROM public.ecr.aws/lambda/microvms:al2023-minimal
RUN {pre}
WORKDIR /app
COPY . .
EXPOSE {port}
CMD {cmd}
"#
    );
    let dockerfile = format!("{}/Dockerfile", dir);
    u::write_str(&dockerfile, &f);
}

fn gen_dockerfile(dir: &str, runtime: &Runtime, bspec: &Build) -> bool {
    let Runtime { port, handler, lang, .. } = runtime;
    match lang.to_lang() {
        Lang::Python => python::gen_dockerfile(dir, handler, port, &bspec.pre),
        _ => gen_generic_dockerfile(dir, handler, port, &bspec.pre),
    }
    true
}

pub async fn build(auth: &Auth, dir: &str, runtime: &Runtime, bspec: &Build) -> BuildStatus {

    let Build { image_name, base_image_arn, build_role_arn, bucket, .. } = bspec;
    let generated = gen_dockerfile(dir, runtime, bspec);

    u::sh(&format!("zip -r -9 {}.zip .", &image_name), dir);

    // copy to s3
    let s3_client = s3::make_client(auth).await;
    let key = format!("{}.zip", &image_name);
    s3::upload_file(&s3_client, &bucket, &key, &key).await;

    let uri = format!("s3://{}/{}", &bucket, &key);

    let client = microvm::make_client(auth).await;
    let mvi = MicroVmImage {
        name: image_name.to_string(),
        base_image_arn: base_image_arn.to_string(),
        build_role_arn: build_role_arn.to_string(),
        uri: uri,
        env: Some(runtime.environment.clone())
    };
    let idem_token = format!("{}_{}", image_name, &auth.name);
    let image_id = mvi.find_or_create(&client, &idem_token).await;

    if generated {
        u::sh("rm -f Dockerfile", dir);
    }
    u::sh("rm -f *.zip", dir);

    BuildStatus {
        path: image_id,
        status: true,
        out: String::from(""),
        err: String::from(""),
    }
}
