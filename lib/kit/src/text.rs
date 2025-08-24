use colored::{
    ColoredString,
    Colorize,
};
use convert_case::{
    Case,
    Casing,
};
use std::collections::HashMap;
use text_placeholder::Template;
use uuid::Uuid;
use rand::{distributions::Alphanumeric, Rng};

pub fn kebab_case(s: &str) -> String {
    s.to_case(Case::Kebab)
}

pub fn snake_case(s: &str) -> String {
    s.to_case(Case::Snake)
}

pub fn pascal_case(s: &str) -> String {
    s.to_case(Case::Pascal)
}

pub fn stencil(s: &str, table: HashMap<&str, &str>) -> String {
    let temp = Template::new(s);
    temp.fill_with_hashmap(&table)
}

pub fn uuid_str() -> String {
    Uuid::new_v4().to_string()
}

pub fn red(s: &str) -> ColoredString {
    s.red()
}

pub fn blue(s: &str) -> ColoredString {
    s.blue()
}

pub fn green(s: &str) -> ColoredString {
    s.green()
}

pub fn mangenta(s: &str) -> ColoredString {
    s.magenta()
}

pub fn randstr() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    s
}
