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

pub async fn publish(auth: &Auth, image_name: &str) {
    let dir = kit::pwd();
    login(auth, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker push {}", &auth.name, image_name);
    u::run(&cmd, &dir);
}
