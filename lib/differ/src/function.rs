use kit as u;
use authorizer::Auth;
use compiler::ConfigSpec;

use crate::{
    aws::lambda,
};
use tabled::{
    Style,
    Table,
    Tabled,
};

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

async fn find(auth: &Auth, fns: Vec<String>) -> Vec<Function> {
    let client = lambda::make_client(auth).await;
    let mut rows: Vec<Function> = vec![];
    for f in fns {
        let tags = lambda::list_tags(client.clone(), &auth.lambda_arn(&f))
            .await
            .unwrap();

        let config = lambda::find_config(&client, &auth.lambda_arn(&f)).await;

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

pub async fn list(auth: &Auth, fns: Vec<String>) {
    let rows = find(auth, fns).await;
    let table = Table::new(rows).with(Style::psql()).to_string();
    println!("{}", table);
}



#[derive(Tabled, Clone, Debug, PartialEq)]
struct Record {
    function: String,
    layer: String,
    current_version: String,
    current_size: String,
    latest_version: i64,
}

fn parse_arn(arn: &str) -> (String, String) {
    let parts: Vec<&str> = arn.split(":").collect();
    (u::nth(parts.clone(), 6), u::nth(parts, 7))
}

pub async fn list_layers(auth: &Auth, fns: Vec<String>) {
    let config = ConfigSpec::new(None);
    let centralized = auth.inherit(config.aws.lambda.layers_profile.to_owned()).await;
    let mut rows: Vec<Record> = vec![];
    let client = lambda::make_client(&centralized).await;
    let cc = lambda::make_client(&auth).await;

    for f in fns {
        let layers = lambda::find_function_layers(&client, &f).await.unwrap();
        for (layer_arn, size) in layers {
            let (name, version) = parse_arn(&layer_arn);
            let latest = lambda::find_latest_version(&cc, &name).await;

            let rec = Record {
                function: f.clone(),
                layer: name,
                current_version: version,
                current_size: u::file_size_human(size as f64),
                latest_version: latest,
            };
            rows.push(rec);
        }
    }
    let table = Table::new(rows).with(Style::psql()).to_string();
    println!("{}", table);
}
