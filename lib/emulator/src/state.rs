use provider::Auth;
use kit as u;

pub async fn run(auth: &Auth) {
    let (key, secret, token) = auth.get_keys().await;
    let env_file = "aws-stepfunctions-local-credentials.txt";
    let region = &auth.region;
    let account = &auth.account;
    let config = format!(
        r"
AWS_ACCOUNT_ID={account}
AWS_DEFAULT_REGION={region}
STEP_FUNCTIONS_ENDPOINT=http://host.docker.internal:8083
LAMBDA_ENDPOINT=http://host.docker.internal:9000
AWS_ACCESS_KEY_ID={key}
AWS_SECRET_ACCESS_KEY={secret}
AWS_SESSIONT_TOKEN={token}
AWS_REGION={region}
",
    );
    let dir = u::pwd();
    u::write_str(env_file, &config);
    println!("Starting states (ASL) emulator");
    u::runcmd_stream(
        "docker run -p 8083:8083 --env-file aws-stepfunctions-local-credentials.txt amazon/aws-stepfunctions-local",
        &dir,
    );
    u::sh("rm -rf aws-stepfunctions-local-credentials.txt", &dir);
}
