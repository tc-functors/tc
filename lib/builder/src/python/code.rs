use kit::sh;

pub fn build(dir: &str, command: &str) -> String {
    let c = format!(r"{}", command);
    sh(&c, dir);
    format!("{}/lambda.zip", dir)
}
