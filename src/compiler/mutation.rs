use super::{MutationSpec, ResolverSpec};
use kit::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Resolver {
    pub kind: String,
    pub name: String,
    pub target: String,
    pub input: String,
    pub output: String,
}

fn kind_of(r: ResolverSpec) -> (String, String) {
    if let Some(f) = r.function {
        (s!("function"), f)
    } else if let Some(e) = r.event {
        (s!("event"), e)
    } else if let Some(t) = r.table {
        (s!("table"), t)
    } else {
        panic!("Invalid Resolver {:?}", r)
    }
}

fn make_resolvers(rspecs: HashMap<String, ResolverSpec>) -> HashMap<String, Resolver> {
    let mut xs: HashMap<String, Resolver> = HashMap::new();
    for (k, r) in rspecs {
        let (kind, target) = kind_of(r.to_owned());
        let resolver = Resolver {
            kind: kind,
            name: k.to_owned(),
            target: target,
            input: r.input,
            output: r.output,
        };
        xs.insert(k, resolver);
    }
    xs
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mutations {
    pub api_name: String,
    pub authorizer: String,
    pub types: HashMap<String, String>,
    pub resolvers: HashMap<String, Resolver>,
    pub types_map: HashMap<String, HashMap<String, String>>,
}

fn make_type(type_name: &str, mappings: HashMap<String, String>) -> String {
    let mut s: String = s!("");
    for (k, v) in mappings {
        s.push_str(&format!("{}: {} ", k, v));
    }
    format!(
        r##"type {type_name} @aws_lambda @aws_iam {{
  {s}
  createdAt: AWSDateTime
  updatedAt: AWSDateTime
}}
"##
    )
}

fn make_query_fields(type_name: &str) -> String {
    format!(
        r#"get{type_name}(id: String!): {type_name}
"#
    )
}

fn make_query_type(fields: &str) -> String {
    format!(
        r#"type Query {{ {fields} }}
"#
    )
}

fn make_sub_fields(type_name: &str, output: String) -> String {
    let sub_name = format!("subscribe{}", &kit::pascal_case(type_name));
    format!(
        r#"{sub_name}(id: String!): {output}
   @aws_subscribe(mutations: ["{type_name}"])
   @aws_lambda @aws_iam
"#
    )
}

fn make_sub_type(fields: &str) -> Option<String> {
    if !fields.is_empty() {
        Some(format!(r#"type Subscription {{ {fields} }}"#))
    } else {
        None
    }
}

fn make_mut_fields(type_name: &str, input: HashMap<String, String>, output: String) -> String {
    let mut s: String = s!("");
    for (k, v) in input {
        s.push_str(&format!("{}: {} ", k, v));
    }
    format!(
        r#"
{type_name}({s}): {output}
@aws_lambda @aws_iam
"#
    )
}

fn make_mut_type(fields: &str) -> String {
    format!(r#"type Mutation {{ {fields} }}"#)
}

type Types = HashMap<String, HashMap<String, String>>;

fn is_subscribable(type_name: &str) -> bool {
    !type_name.ends_with("Input") && type_name != "Event"
}

fn make_types(types: Types, resolvers: HashMap<String, ResolverSpec>) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let mut query_fields: String = s!("");
    for (type_name, mappings) in types.clone() {
        h.insert(s!(&type_name), make_type(&type_name, mappings));
        let f = make_query_fields(&type_name);
        query_fields.push_str(&f);
    }

    h.insert(s!("Query"), make_query_type(&query_fields));

    let mut mut_fields: String = s!("");
    for (type_name, resolver) in resolvers.clone() {
        let input = types.get(&resolver.input);
        match input {
            Some(it) => {
                let f = make_mut_fields(&type_name, it.clone(), resolver.output);
                mut_fields.push_str(&f);
            }
            None => (),
        }
    }
    h.insert(s!("Mutation"), make_mut_type(&mut_fields));

    let mut sub_fields: String = s!("");
    for (type_name, resolver) in resolvers {
        let ResolverSpec {
            subscribe, output, ..
        } = resolver;
        if is_subscribable(&type_name) && subscribe {
            let f = make_sub_fields(&type_name, output.to_owned());
            sub_fields.push_str(&f);
        }
    }

    let subs = make_sub_type(&sub_fields);
    if let Some(s) = subs {
        h.insert(s!("Subscription"), s);
    }
    h
}

fn augment_types(mut given: Types) -> Types {
    let mut event: HashMap<String, String> = HashMap::new();
    event.insert(s!("detail"), s!("String"));
    let mut types: HashMap<String, HashMap<String, String>> = HashMap::new();
    types.insert(s!("Event"), event);
    given.extend(types);
    given
}

pub fn make(namespace: &str, some_mutatations: Option<MutationSpec>) -> Option<Mutations> {
    match some_mutatations {
        Some(ms) => {
            let types = augment_types(ms.types.to_owned());
            let m = Mutations {
                api_name: format!("{}_{{sandbox}}", namespace),
                authorizer: ms.authorizer.to_owned(),
                types: make_types(types.to_owned(), ms.resolvers.to_owned()),
                resolvers: make_resolvers(ms.resolvers),
                types_map: types,
            };
            Some(m)
        }
        None => None,
    }
}

pub fn print_graphql(types: &HashMap<String, String>) {
    for (_, v) in types {
        println!("{}", v)
    }
}
