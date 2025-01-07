mod core;
mod http;
mod io;
mod json;
mod pprint;
mod text;
mod time;

pub use self::core::*;
pub use self::http::*;
pub use self::io::*;
pub use self::json::*;
pub use self::pprint::*;
pub use self::text::*;
pub use self::time::*;

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
