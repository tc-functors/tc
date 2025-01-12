use aws::lambda;
use aws::Env;

pub async fn invoke(env: &Env, name: &str, payload: &str) {
    let client = lambda::make_client(env).await;
    println!("Invoking function {}", name);
    let _ = lambda::invoke(client, name, payload).await;
}
