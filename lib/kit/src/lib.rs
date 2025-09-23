mod core;
mod crypto;
mod github;
mod http;
mod io;
mod json;
mod pprint;
mod text;
mod time;
mod git;

pub use self::{
    core::*,
    crypto::*,
    github::*,
    http::*,
    io::*,
    json::*,
    pprint::*,
    text::*,
    time::*,
    git::*,
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
