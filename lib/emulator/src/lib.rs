pub mod lambda;
pub mod sfn;
pub mod shell;

use kit as u;
use provider::Env;

fn as_dev_layers(layers: Vec<String>) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    for layer in layers {
        xs.push(format!("{}-dev", &layer));
    }
    xs
}

pub async fn shell(env: &Env, dev: bool) {
    let dir = u::pwd();
    let function = compiler::current_function(&dir);
    match function {
        Some(f) => {
            let layers = if dev {
                as_dev_layers(f.runtime.layers)
            } else {
                f.runtime.layers
            };

            shell::run(
                env,
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

pub async fn lambda(env: &Env, dev: bool) {
    let dir = u::pwd();
    let function = compiler::current_function(&dir);
    match function {
        Some(f) => {
            let layers = if dev {
                as_dev_layers(f.runtime.layers)
            } else {
                f.runtime.layers
            };

            lambda::run(
                env,
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
