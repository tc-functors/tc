mod core;
mod crypto;
mod git;
mod github;
mod http;
mod io;
mod json;
mod memo;
mod pprint;
mod text;
mod time;

pub use self::{
    core::*,
    crypto::*,
    git::*,
    github::*,
    http::*,
    io::*,
    json::*,
    memo::*,
    pprint::*,
    text::*,
    time::*,
};

#[macro_export]
macro_rules! s {
    ($($e:expr),* $(,)?) => {
        {
            let mut string: String = String::new();
            $(
                let add: &str = &$e.to_string();
                string.push_str(add);
            )*
                string
        }
    };
}

#[macro_export]
macro_rules! ln {
    () => {
        println!()
    };
}

#[macro_export]
macro_rules! v {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

// --- DEMO: intentional house-style violations (do NOT merge) ---
// Meant to trip the conformance gate + AI reviewers:
//  - adds `anyhow` (deliberately absent from this codebase)
//  - a Result-returning API in ordinary code
//  - logic in an impl method instead of a terse free function
use anyhow::Result;

pub struct Greeter {
    pub name: String,
}

impl Greeter {
    pub fn greeting(&self) -> Result<String> {
        Ok(format!("hello {}", self.name))
    }
}
