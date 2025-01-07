use aws::Env;
use aws::{lambda, layer};
use kit as u;
use tabled::{Style, Table, Tabled};

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

pub async fn list(env: &Env, fns: Vec<String>) {
    let mut rows: Vec<Record> = vec![];
    let client = lambda::make_client(env).await;
    let cc = layer::make_client(&env).await;

    for f in fns {
        let layers = lambda::find_function_layers(&client, &f).await.unwrap();
        for (layer_arn, size) in layers {
            let (name, version) = parse_arn(&layer_arn);
            let latest = layer::find_latest_version(&cc, &name).await;

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
