pub mod lambda;
pub mod sfn;
pub mod shell;

use crate::compiler;
use aws::Env;
use kit as u;

pub async fn shell(env: &Env) {
    let dir = u::pwd();
    let function = compiler::current_function(&dir);
    match function {
        Some(f) => {
            shell::run(
                env,
                &f.name,
                &f.runtime.lang,
                &f.runtime.handler,
                f.runtime.layers,
            )
            .await;
        }
        None => (),
    }
}

pub async fn lambda(env: &Env) {
    let dir = u::pwd();
    let function = compiler::current_function(&dir);
    match function {
        Some(f) => {
            lambda::run(
                env,
                &f.name,
                &f.runtime.lang,
                f.runtime.layers,
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
