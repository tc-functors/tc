use crate::compiler::FunctionSpec;
use kit as u;

pub async fn test(dir: &str, function: FunctionSpec) {
    let tasks = function.tasks;
    let test_task = tasks.get("test");
    let task = match test_task {
        Some(t) => t,
        None => "tc invoke --local",
    };
    u::runcmd_stream(&task, dir);
}
