use kit as u;

pub fn run() {
    let env_file = "aws-stepfunctions-local-credentials.txt";
    let config = format!(
        r"
AWS_DEFAULT_REGION=us-west-2
STEP_FUNCTIONS_ENDPOINT=http://host.docker.internal:8083
"
    );
    let dir = u::pwd();
    u::write_str(env_file, &config);
    u::runcmd_stream("docker run -p 8083:8083 --env-file aws-stepfunctions-local-credentials.txt amazon/aws-stepfunctions-local", &dir);
    u::sh("rm -f aws-stepfunctions-local-credentials.txt", &dir);
}
