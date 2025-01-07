use aws::layer;
use aws::Env;
use colored::Colorize;
use kit as u;
use std::collections::HashMap;
use std::env;
use tabled::{Style, Table};

pub fn should_split(dir: &str) -> bool {
    let zipfile = "deps.zip";
    let size;
    if u::path_exists(dir, zipfile) {
        size = u::path_size(dir, zipfile);
    } else {
        return false;
    }
    size >= 70000000.0
}

pub async fn publish(env: &Env, lang: &str, layer_name: &str, zipfile: &str) {
    let client = layer::make_client(&env).await;
    if u::file_exists(zipfile) {
        let bar = kit::progress();
        let prefix = format!("Publishing {}", layer_name.blue());
        bar.set_prefix(prefix);
        bar.inc(20);
        let version = layer::publish(&client, layer_name, zipfile, lang).await;
        layer::add_permission(&client, layer_name, version).await;
        bar.set_message(format!("(version: {})", version));
        bar.finish();
    }
}

async fn layer_arn(env: &Env, name: &str, version: Option<String>) -> String {
    match version {
        Some(v) => {
            let layer = format!("{}:{}", name, &v);
            env.layer_arn(&layer)
        }
        None => {
            let client = layer::make_client(&env).await;
            layer::find_version(client, name).await.unwrap()
        }
    }
}

pub async fn promote(env: &Env, layer_name: &str, lang: &str, version: Option<String>) {
    let client = layer::make_client(&env).await;
    let dev_layer_name = format!("{}-dev", layer_name);

    let dev_layer_arn = layer_arn(&env, &dev_layer_name, version).await;
    println!("Promoting {}", dev_layer_arn);
    let maybe_url = layer::get_code_url(&client, &dev_layer_arn).await;

    match maybe_url {
        Some(url) => {
            let tmp_path = env::temp_dir();
            let tmp_dir = tmp_path.to_string_lossy();
            let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
            u::download(&url, HashMap::new(), &tmp_zip_file).await;

            let size = u::file_size(&tmp_zip_file);
            println!(
                "Publishing {} ({})",
                layer_name,
                u::file_size_human(size).green()
            );

            let version = layer::publish(&client, layer_name, &tmp_zip_file, lang).await;

            println!("Published {}:{} (stable)", layer_name, version);
            layer::add_permission(&client, layer_name, version).await;
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => panic!("Layer promotion failed"),
    }
}

pub async fn publish_as_dev(env: &Env, layer_name: &str, lang: &str) {
    let client = layer::make_client(&env).await;
    let layer_arn = env.resolve_layer(&layer_name).await;
    let maybe_url = layer::get_code_url(&client, &layer_arn).await;
    match maybe_url {
        Some(url) => {
            let tmp_path = env::temp_dir();
            let tmp_dir = tmp_path.to_string_lossy();
            let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
            let dev_layer_name = format!("{}-dev", &layer_name);

            println!("Publishing {} ", &dev_layer_name);
            u::download(&url, HashMap::new(), &tmp_zip_file).await;
            let version = layer::publish(&client, &dev_layer_name, &tmp_zip_file, lang).await;

            println!("Published {}:{}", &dev_layer_name, version);
            layer::add_permission(&client, &dev_layer_name, version).await;
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => panic!("Layer publishing failed"),
    }
}

pub async fn list(env: &Env, layer_names: Vec<String>) -> String {
    let client = layer::make_client(&env).await;
    let layers = layer::list(client, layer_names).await;
    Table::new(layers).with(Style::psql()).to_string()
}
