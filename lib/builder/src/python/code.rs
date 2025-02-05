use kit::sh;

pub fn build(dir: &str, command: &str, trace: bool) -> String {
    let c = format!(r"{}", command);
    if trace {
        println!("{}", &c);
    }
    sh(&c, dir);
    format!("{}/lambda.zip", dir)
}
