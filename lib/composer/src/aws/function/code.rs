
use compiler::FunctionSpec;
use kit as u;
use std::collections::HashMap;

fn gen_handler(dir: &str, name: &str, content: &str) {
    let path = format!("{}/{}.py", dir, name);
    u::write_str(&path, content);
}

pub fn scaffold(namespace: &str, functions: HashMap<String, FunctionSpec>) {
    let dir = format!("/tmp/tc-fns/{}", &namespace);
    u::mkdir(&dir);
    for (name, f) in functions {
        if let Some(code) = f.runtime.unwrap().code {
            gen_handler(&dir, &name, &code);
        }
    }
}
