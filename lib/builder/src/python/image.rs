
pub fn build(_dir: &str, _name: &str) -> String {
    // generate dockerfile
    //sh(&format!("docker build --no-cache  --ssh default . -t {}", name));
    format!("docker")
}
