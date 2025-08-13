mod function;
mod topology;

pub fn scaffold(kind: &str) {
    match kind {
        "function" => function::scaffold(),
        "topology" => topology::scaffold(),
        _ => ()
    }
}
