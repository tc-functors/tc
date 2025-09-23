mod core;
mod crypto;
mod git;
mod github;
mod http;
mod io;
mod json;
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
