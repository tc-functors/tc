use colored::Colorize;
use kit as u;
use authorizer::Auth;
use crate::aws::lambda;

use std::{
    collections::HashMap,
};

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

pub async fn publish(
    auth: &Auth,
    lang: &str,
    layer_name: &str,
    zipfile: &str
) {

    println!("Using profile {}", auth.name);
    let client = lambda::make_client(auth).await;

    if u::file_exists(zipfile) {
        println!("Publishing {}", layer_name.blue());
        let version = lambda::publish(&client, layer_name, zipfile, lang).await;
        lambda::add_permission(&client, layer_name, version).await;
        println!("(version: {})", version);
    }
}

async fn layer_arn(auth: &Auth, name: &str, version: Option<String>) -> String {
    match version {
        Some(v) => {
            let layer = format!("{}:{}", name, &v);
            auth.layer_arn(&layer)
        }
        None => {
            let client = lambda::make_client(&auth).await;
            lambda::find_layer_version(&client, name).await.unwrap()
        }
    }
}

pub async fn promote(
    auth: &Auth,
    layer_name: &str,
    lang: &str,
    version: Option<String>
) {
    let client = lambda::make_client(&auth).await;
    let dev_layer_name = format!("{}-dev", layer_name);

    let dev_layer_arn = layer_arn(&auth, &dev_layer_name, version).await;
    println!("Promoting {}", dev_layer_arn);
    let maybe_url = lambda::get_code_url(&client, &dev_layer_arn).await;

    match maybe_url {
        Some(url) => {
            let tmp_path = std::env::temp_dir();
            let tmp_dir = tmp_path.to_string_lossy();
            let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
            u::download(&url, HashMap::new(), &tmp_zip_file).await;

            let size = u::file_size(&tmp_zip_file);
            println!(
                "Publishing {} ({})",
                layer_name,
                u::file_size_human(size).green()
            );

            let version = lambda::publish(&client, layer_name, &tmp_zip_file, lang).await;

            println!("Published {}:{} (stable)", layer_name, version);
            lambda::add_permission(&client, layer_name, version).await;
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => panic!("Layer promotion failed"),
    }
}

pub async fn publish_as_dev(auth: &Auth, layer_name: &str, lang: &str) {
    let client = lambda::make_client(auth).await;
    let layer_arn = lambda::find_layer_version(&client, layer_name).await.unwrap();
    let maybe_url = lambda::get_code_url(&client, &layer_arn).await;
    match maybe_url {
        Some(url) => {
            let tmp_path = std::env::temp_dir();
            let tmp_dir = tmp_path.to_string_lossy();
            let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
            let dev_layer_name = format!("{}-dev", &layer_name);

            println!("Publishing {} ", &dev_layer_name);
            u::download(&url, HashMap::new(), &tmp_zip_file).await;
            let version = lambda::publish(&client, &dev_layer_name, &tmp_zip_file, lang).await;

            println!("Published {}:{}", &dev_layer_name, version);
            lambda::add_permission(&client, &dev_layer_name, version).await;
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => panic!("Layer publishing failed"),
    }
}

pub async fn download(auth: &Auth, name: &str) {
    let client = lambda::make_client(&auth).await;

    let layer = lambda::find_layer_version(&client, name).await.unwrap();
    println!("Resolving layer: {}", &layer);
    let target_dir = format!("{}/layer", &u::pwd());
    u::sh(&format!("rm -rf {}", &target_dir), &u::pwd());


    let maybe_url = lambda::get_code_url(&client, &layer).await;

    match maybe_url {
        Some(url) => {
            let tmp_path = std::env::temp_dir();
            let tmp_dir = tmp_path.to_string_lossy();
            let tmp_zip_file = format!("{}/{}.zip", tmp_dir, u::uuid_str());
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
            println!("Downloading to layer/ dir");
            u::download(&url, HashMap::new(), &tmp_zip_file).await;
            u::sh(
                &format!("unzip -o {} -d {}", tmp_zip_file, target_dir),
                &tmp_dir,
            );
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => (),
    }
}
