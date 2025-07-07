use composer::Function;
use kit as u;

pub async fn test(dir: &str, _function: Function) {
    u::runcmd_stream("poetry test", dir);
}
