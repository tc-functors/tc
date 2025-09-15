use colored::{
    ColoredString,
    Colorize,
};
use convert_case::{
    Case,
    Casing,
};
use regex::Regex;
use std::collections::HashMap;
use text_placeholder::Template;

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

pub fn find_matches(s: &str, pattern: &str) -> Vec<String> {

    let re = Regex::new(pattern).unwrap();

    let mut res: Vec<String> = vec![];
    let xs: Vec<_> = re.find_iter(s).map(|mat| mat.as_str()).collect();
    for x in xs {
        let parts: Vec<&str> = x.split("\n").collect();
        res.push(parts.clone().first().unwrap().to_string());
    }
    res

}
