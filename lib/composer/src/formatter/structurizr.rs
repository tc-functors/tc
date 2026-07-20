use crate::Topology;
use std::collections::HashMap;

fn make_model() -> String {
    format!(r#"
        u = person "User"
        ss = softwareSystem "Software System"

        u -> ss "Uses"
"#)
}

fn make_views() -> String {
    format!(r#"
        systemContext ss {{
            include *
            autoLayout
        }}
"#)
}

fn build(model: &str, views: &str) -> String {
    format!(r#"
workspace "tc" "Description" {{
   model {{
      {model}
   }}
   views {{
      {views}
   }}
}}
"#)

}

pub fn pprint(_topology: &Topology) {
    let model = make_model();
    let views = make_views();
    let out = build(&model, &views);
    println!("{}", &out);
}


fn make_root_model(topologies: &HashMap<String, Topology>) -> String {
    let mut containers: String = "".to_string();
    for (name, _topology) in topologies {
        let s = format!(r#"
        {name} = container "{name}""#);
        containers.push_str(&s);
    }

    format!(r#"
        user = person "User" "User"

        system = softwareSystem "System" {{
           {containers}
        }}
    "#)

}

fn make_root_views() -> String {
    format!(r#"
        systemContext system "System" "System" {{
            include *
            autoLayout
        }}

        container system "Containers" "Container diagram" {{
            include *
            autoLayout
        }}
"#)
}

pub fn pprint_recursive(topologies: &HashMap<String, Topology>) {
    let model = make_root_model(topologies);
    let views = make_root_views();
    let out = build(&model, &views);
    println!("{}", &out);
}
