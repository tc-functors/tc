mod core;
mod http;
mod io;
mod json;
mod pprint;
mod text;
mod time;
mod github;

pub use self::{
    core::*,
    http::*,
    io::*,
    json::*,
    pprint::*,
    text::*,
    time::*,
    github::*
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
