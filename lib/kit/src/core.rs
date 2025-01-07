use itertools::Itertools;
use std::collections::HashMap;

pub fn s(st: &str) -> String {
    String::from(st)
}

pub fn empty() -> String {
    s("")
}

pub fn triml(input: &str) -> &str {
    input
        .strip_prefix("\r\n")
        .or(input.strip_prefix('\n'))
        .unwrap_or(input)
}

pub fn trim(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix('\n'))
        .unwrap_or(input)
}

pub fn strip(input: &str, suffix: &str) -> String {
    input
        .strip_suffix(suffix)
        .or(input.strip_suffix('/'))
        .unwrap_or(input)
        .to_string()
}

pub fn nth(xs: Vec<&str>, n: u8) -> String {
    xs.clone()
        .into_iter()
        .nth(n.into())
        .unwrap_or_default()
        .to_string()
}

pub fn lastn(xs: Vec<&str>, n: usize) -> Vec<&str> {
    xs.clone().into_iter().rev().take(n).collect()
}

pub fn split_oncel(s: &str) -> &str {
    match s.split_once("-") {
        Some((_k, v)) => &v,
        _ => "",
    }
}

pub fn splitf(s: &str) -> &str {
    let parts: Vec<&str> = s.split("-").collect();
    parts.clone().first().unwrap()
}

pub fn splitl(s: &str) -> &str {
    let parts: Vec<&str> = s.split("-").collect();
    parts.clone().last().unwrap()
}

pub fn split_last(s: &str, delimiter: &str) -> String {
    let parts: Vec<&str> = s.split(delimiter).collect();
    parts.clone().last().unwrap().to_string()
}

pub fn split_first(s: &str, delimiter: &str) -> String {
    let parts: Vec<&str> = s.split(delimiter).collect();
    parts.clone().first().unwrap().to_string()
}

pub fn second(s: &str, delimiter: &str) -> String {
    let parts: Vec<&str> = s.split(delimiter).collect();
    parts.into_iter().nth(1).unwrap_or_default().to_string()
}

pub fn split_lines(s: &str) -> Vec<&str> {
    let parts: Vec<&str> = s.split("\n").collect();
    parts
}

// option

pub fn option_exists(opt: Option<String>) -> bool {
    match opt {
        Some(_) => true,
        None => false,
    }
}

pub fn unwrap(st: Option<String>) -> String {
    match st {
        Some(k) => k,
        _ => empty(),
    }
}

pub fn maybe_str(s: Option<&str>) -> String {
    match s {
        Some(p) => p.to_string(),
        None => "".to_string(),
    }
}

pub fn maybe_string(s: Option<String>, default: &str) -> String {
    match s {
        Some(p) => p,
        None => default.to_string(),
    }
}

pub fn maybe_int(v: Option<i32>, default: i32) -> i32 {
    match v {
        Some(i) => i,
        None => default,
    }
}

pub fn maybe_hashmap(
    h: Option<HashMap<String, String>>,
    default: HashMap<String, String>,
) -> HashMap<String, String> {
    match h {
        Some(v) => v,
        None => default,
    }
}

pub fn inc(n: &mut isize) {
    *n += 1;
}

pub fn _remove_suffix<'a>(s: &'a str, suffix: &str) -> &'a str {
    match s.strip_suffix(suffix) {
        Some(s) => s,
        None => s,
    }
}

pub fn kv(k: &str, v: &str) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(k.to_string(), v.to_string());
    h
}

pub fn capitalize(s: &str) -> String {
    format!("{}{}", (&s[..1].to_string()).to_uppercase(), &s[1..])
}

pub fn vec_to_str(v: Vec<&str>) -> String {
    v.join(" ")
}

pub fn vecs_str(v: Vec<String>) -> String {
    v.join(" ")
}

pub fn safe_unwrap(opt: Option<&String>) -> String {
    match opt {
        Some(s) => s.to_string(),
        None => "".to_string(),
    }
}

pub fn uniq(xs: Vec<String>) -> Vec<String> {
    xs.into_iter().unique().into_iter().collect::<Vec<String>>()
}

pub fn abbreviate(s: &str, delimiter: &str) -> String {
    let parts: Vec<&str> = s.split(delimiter).collect();
    let mut chs: Vec<char> = vec![];
    for part in parts {
        let ch = part.chars().nth(0).unwrap();
        chs.push(ch);
    }
    chs.iter().collect()
}

pub fn maybe_vec_string(t: Option<Vec<String>>) -> String {
    match t {
        Some(s) => match s.first() {
            Some(st) => st.to_string(),
            None => "".to_string(),
        },
        None => "".to_string(),
    }
}
