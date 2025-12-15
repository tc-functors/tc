use colored::Colorize;
use composer::Function;
use configurator::Config;
use kit as u;
use provider::{
    Auth,
    aws,
};
use std::collections::HashMap;

pub fn render_uri(uri: &str, repo: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("repo", repo);
    u::stencil(uri, table)
}

pub async fn run(auth: &Auth, dir: &str, function: &Function, _shell: bool) {

    let Function { name, .. } = function;

    let config = Config::new();

    let repo = match std::env::var("TC_ECR_REPO") {
        Ok(r) => &r.to_owned(),
        Err(_) => &config.aws.ecr.repo,
    };

    let uri =  match std::env::var("CODE_IMAGE_URI") {
        Ok(p) => p,
        Err(_) => function.runtime.uri.clone()
    };
    let code_image_uri = render_uri(&uri, repo);

    let maybe_cfg_profile = config.aws.lambda.layers_profile.clone();
    let auth = match maybe_cfg_profile {
        Some(p) => auth.assume(Some(p.clone()), config.role_to_assume(Some(p))).await,
        None => auth.clone()
    };

    println!("ecr login {}", &auth.name);
    aws::ecr::login(&auth, &dir).await;

    println!(
        "Building emulator: {} from {}",
        &name.cyan(),
        &code_image_uri
    );

    let b_cmd = format!("docker run --rm -it --entrypoint bash {}", &code_image_uri);
    u::runcmd_stream(&b_cmd, dir);
}
