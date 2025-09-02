use chksum::sha1;
use rand::{
    Rng,
    distributions::Alphanumeric,
};
use std::fs::read_dir;
use uuid::Uuid;

pub fn checksum_dir(dir: &str) -> String {
    let readdir = read_dir(dir).unwrap();
    let digest = sha1::chksum(readdir).unwrap();
    digest.to_hex_lowercase()
}

pub fn checksum_str(s: &str) -> String {
    let digest = sha1::chksum(s).unwrap();
    digest.to_hex_lowercase()
}

pub fn randstr() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    s
}

pub fn uuid_str() -> String {
    Uuid::new_v4().to_string()
}
