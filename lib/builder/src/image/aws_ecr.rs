use kit as u;

use authorizer::Auth;

fn get_host(auth: &Auth) -> String {
    format!("{}.dkr.ecr.{}.amazonaws.com", auth.account, auth.region)
}

pub async fn login(auth: &Auth, dir: &str) {
    let cmd = format!(
        "AWS_PROFILE={} aws ecr get-login-password --region {} | docker login --username AWS --password-stdin {}",
        &auth.name,
        auth.region,
        get_host(auth)
    );
    u::run(&cmd, dir);
}
