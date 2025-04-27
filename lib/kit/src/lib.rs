mod core;
mod http;
mod io;
mod json;
mod pprint;
mod text;
mod time;

pub use self::{
    core::*,
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
