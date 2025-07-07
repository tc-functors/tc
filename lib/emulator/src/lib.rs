mod aws;
pub mod function;
pub mod sfn;
pub mod shell;

use authorizer::Auth;
use kit as u;

fn as_dev_layers(layers: Vec<String>) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    for layer in layers {
        xs.push(format!("{}-dev", &layer));
    }
    xs
}

pub async fn shell(auth: &Auth, dev: bool) {
    let dir = u::pwd();
    let function = composer::current_function(&dir);
    match function {
        Some(f) => {
            let layers = if dev {
                as_dev_layers(f.runtime.layers)
            } else {
                f.runtime.layers
            };

            shell::run(
                auth,
                &f.name,
                &f.runtime.lang.to_str(),
                &f.runtime.handler,
                layers,
            )
            .await;
        }
        None => (),
    }
}

pub async fn lambda(auth: &Auth, dev: bool) {
    let dir = u::pwd();
    let function = composer::current_function(&dir);
    match function {
        Some(f) => {
            let layers = if dev {
                as_dev_layers(f.runtime.layers)
            } else {
                f.runtime.layers
            };

            function::run(
                auth,
                &f.name,
                &f.runtime.lang.to_str(),
                layers,
                &f.runtime.handler,
            )
            .await;
        }
        None => (),
    }
}

pub async fn sfn() {
    sfn::run();
}
