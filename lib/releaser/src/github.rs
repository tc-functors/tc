use kit as u;
use kit::*;
use serde_derive::Deserialize;
use std::{collections::HashMap, env, fs};

pub fn arch_os() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    format!("{}-{}", arch, os)
}

pub fn replace_exe(new_path: &str) {
    self_replace::self_replace(&new_path).unwrap();
    fs::remove_file(&new_path).unwrap();
}

#[derive(Deserialize, Clone, Debug)]
struct Asset {
    id: u64,
    browser_download_url: String,
    size: f64,
    name: String,
}

impl Asset {
    fn version(&self) -> String {
        nth(
            self.browser_download_url.split("/").into_iter().collect(),
            7,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Github {
    pub repo: String,
}

impl Github {
    pub fn init(repo: &str) -> Github {
        Github {
            repo: String::from(repo),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        //h.insert(s!("authorization"), format!("Bearer {}", self.token));
        h.insert(s!("accept"), s!("application/vnd.github+json"));
        h.insert(s!("x-github-api-version"), s!("2022-11-28"));
        h.insert(
            s!("user-agent"),
            s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
        );
        h
    }

    fn with_headers(&self, key: &str, val: &str) -> HashMap<String, String> {
        let mut h = self.headers();
        h.insert(s!(key), s!(val));
        h
    }

    fn url(&self, path: &str) -> String {
        format!(
            "https://api.github.com/repos/tc-functors/{}{}",
            self.repo, path
        )
    }

    async fn latest_release_assets(&self) -> Vec<Asset> {
        let res = u::http_get(&self.url("/releases/latest"), self.headers()).await;
        let assets = &res["assets"];
        let ats: Vec<Asset> = serde_json::from_value(assets.clone()).unwrap();
        ats
    }

    async fn release_assets_by_tag(&self, tag: &str) -> Vec<Asset> {
        let path = format!("/releases/tags/{}", tag);
        let res = u::http_get(&self.url(&path), self.headers()).await;
        let assets = &res["assets"];
        let ats: Vec<Asset> = serde_json::from_value(assets.clone()).unwrap();
        ats
    }

    async fn download_asset(&self, asset_name: &str, outfile: &str) {
        let assets = self.latest_release_assets().await;
        let headers = self.with_headers("accept", "application/octet-stream");
        for asset in assets {
            let id = &asset.id;
            if &asset.name == asset_name {
                println!(
                    "Upgrading to {} ({}) ref:{}",
                    &asset.version(),
                    file_size_human(asset.size),
                    id
                );
                let path = format!("/releases/assets/{}", id);
                u::download(&self.url(&path), headers.clone(), outfile).await;
                replace_exe(outfile);
                println!("Everything you can imagine is real - Pablo Picasso");
            }
        }
    }

    async fn download_asset_by_tag(&self, asset_name: &str, outfile: &str, tag: &str) {
        let assets = self.release_assets_by_tag(tag).await;
        let headers = self.with_headers("accept", "application/octet-stream");
        for asset in assets {
            let id = &asset.id;
            if &asset.name == asset_name {
                println!(
                    "Upgrading to {} ({}) ref:{}",
                    &asset.version(),
                    file_size_human(asset.size),
                    id
                );
                let path = format!("/releases/assets/{}", id);
                u::download(&self.url(&path), headers.clone(), outfile).await;
                replace_exe(outfile);
                println!("Everything you can imagine is real - Pablo Picasso");
            }
        }
    }
}

pub async fn get_release_id(repo: &str, tag: Option<String>) -> Option<String> {
    let gh = Github::init(repo);
    let rel_name = "tc-x86_64-linux";
    let assets = match tag {
        Some(t) => gh.release_assets_by_tag(&t).await,
        None => gh.latest_release_assets().await,
    };
    for asset in assets {
        if &asset.name == rel_name {
            return Some(asset.id.to_string());
        }
    }
    None
}

pub async fn self_upgrade(repo: &str, tag: Option<String>) {
    let gh = Github::init(repo);
    let arch_os = arch_os();
    let name = match arch_os.as_str() {
        "x86_64-linux" => "tc-x86_64-linux",
        "x86_64-macos" => "tc-x86_64-macos",
        "aarch64-macos" => "tc-aarch64-macos",
        _ => panic!("unknown os {}", arch_os),
    };
    println!(
        "Fetching release from https://github.com/tc-functors/{}",
        repo
    );
    match tag {
        Some(t) => gh.download_asset_by_tag(name, "/tmp/tc", &t).await,
        None => gh.download_asset(name, "/tmp/tc").await,
    }
}
