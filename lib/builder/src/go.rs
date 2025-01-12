use kit as u;

pub fn pack(dir: &str, _command: &str) {
    u::sh("rm -f lambda.zip", dir);
    let cmd = "GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -tags lambda.norpc -o bootstrap main.go && zip lambda.zip bootstrap";
    u::sh(&cmd, dir);
}
