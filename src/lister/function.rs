use aws::lambda;
use aws::Env;
use kit as u;
use tabled::{Style, Table, Tabled};

#[derive(Tabled, Clone, Debug, PartialEq)]
struct Function {
    name: String,
    code_size: String,
    timeout: i32,
    mem: i32,
    revision: String,
    updated: String,
    tc_version: String,
}

async fn find(env: &Env, fns: Vec<String>) -> Vec<Function> {
    let client = lambda::make_client(env).await;
    let mut rows: Vec<Function> = vec![];
    for f in fns {
        let tags = lambda::list_tags(client.clone(), &env.lambda_arn(&f))
            .await
            .unwrap();

        let config = lambda::find_config(&client, &env.lambda_arn(&f)).await;

        match config {
            Some(cfg) => {
                let row = Function {
                    name: f,
                    code_size: u::file_size_human(cfg.code_size as f64),
                    timeout: cfg.timeout,
                    mem: cfg.mem_size,
                    revision: cfg.revision,
                    tc_version: u::safe_unwrap(tags.get("tc_version")),
                    updated: u::safe_unwrap(tags.get("updated_at")),
                };
                rows.push(row);
            }
            None => (),
        }
    }
    rows
}

pub async fn list(env: &Env, fns: Vec<String>) {
    let rows = find(env, fns).await;
    let table = Table::new(rows).with(Style::psql()).to_string();
    println!("{}", table);
}
