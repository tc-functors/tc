mod python;
mod ruby;
mod aws_lambda;
use colored::Colorize;
use kit as u;
use compiler::{LangRuntime, Lang};
use compiler::spec::BuildOutput;
use authorizer::Auth;
use std::collections::HashMap;

fn should_split(dir: &str) -> bool {
    let zipfile = "deps.zip";
    let size;
    if u::path_exists(dir, zipfile) {
        size = u::path_size(dir, zipfile);
    } else {
        return false;
    }
    size >= 60000000.0
}

fn split(dir: &str) {

    let zipfile = format!("{}/deps.zip", dir);
    let size;
    if u::file_exists(&zipfile) {
        size = u::file_size(&zipfile);
    } else {
        panic!("No zip found");
    }
    if size >= 60000000.0 {
        let cmd = format!("zipsplit {} -n 50000000", zipfile);
        u::runcmd_stream(&cmd, dir);
    }
}

pub async fn do_publish(
    auth: &Auth,
    lang: &str,
    layer_name: &str,
    zipfile: &str
) {

    println!("Using profile {}", auth.name);
    let client = aws_lambda::make_client(auth).await;

    if u::file_exists(zipfile) {
        println!("Publishing {}", layer_name.blue());
        let version = aws_lambda::publish(&client, layer_name, zipfile, lang).await;
        aws_lambda::add_permission(&client, layer_name, version).await;
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
            let client = aws_lambda::make_client(&auth).await;
            aws_lambda::find_layer_version(&client, name).await.unwrap()
        }
    }
}

pub async fn publish(auth: &Auth, build: &BuildOutput) {
    let BuildOutput { dir, runtime, name, artifact, .. } = build;
    let lang = runtime.to_str();
    if should_split(&dir) {
        println!("Split layer ... {}", &name);
        split(&dir);
        if u::path_exists(dir, "deps1.zip") {
            do_publish(auth, &lang, &format!("{}-0-dev", &name), "deps1.zip").await;
        }
        if u::path_exists(dir, "deps2.zip") {
            do_publish(auth, &lang, &format!("{}-1-dev", &name), "deps2.zip").await;
        }
        if u::path_exists(dir, "deps3.zip") {
            do_publish(auth, &lang, &format!("{}-2-dev", &name), "deps3.zip").await;
        }

    } else {
        let layer_name = format!("{}-dev", &name);
        do_publish(auth, &lang, &layer_name, &artifact).await;
    }
}

pub async fn promote(
    auth: &Auth,
    layer_name: &str,
    lang: &str,
    version: Option<String>
) {
    let client = aws_lambda::make_client(&auth).await;
    let dev_layer_name = format!("{}-dev", layer_name);

    let dev_layer_arn = layer_arn(&auth, &dev_layer_name, version).await;
    println!("Promoting {}", dev_layer_arn);
    let maybe_url = aws_lambda::get_code_url(&client, &dev_layer_arn).await;

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

            let version = aws_lambda::publish(&client, layer_name, &tmp_zip_file, lang).await;

            println!("Published {}:{} (stable)", layer_name, version);
            aws_lambda::add_permission(&client, layer_name, version).await;
            u::sh(&format!("rm -rf {}", tmp_zip_file), &u::pwd());
        }
        None => panic!("Layer promotion failed"),
    }
}

pub fn build(dir: &str, name: &str, langr: &LangRuntime) -> String {

    match langr.to_lang() {
        Lang::Python => python::build(dir, name, langr),
        Lang::Ruby => ruby::build(dir, name),
        _ => todo!()
    }

}
