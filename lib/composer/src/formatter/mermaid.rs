use crate::Topology;
use kit as u;
use std::collections::HashMap;

struct Target {
    from: String,
    to: String,
}

fn name_only(s: &str) -> String {
    if s.starts_with("{{") {
        u::second(&s, "_")
    } else {
        s.to_string()
    }
}

fn build_flow(topology: &Topology) -> String {
    let mut targets: Vec<Target> = vec![];
    let mut s: String = String::from("");

    let Topology {
        routes,
        events,
        channels,
        mutations,
        functions,
        queues,
        namespace,
        flow,
        ..
    } = topology;

    if routes.len() > 0 {
        let mut rs = format!("subgraph routes");
        for (name, route) in routes {
            let name = if name.starts_with("/") {
                let gname = name.replace("{", "").replace("}", "").replace("/", "_");
                format!(
                    "route_{}{{{}}}",
                    gname,
                    route.path.replace("{", "").replace("}", "")
                )
            } else {
                name.to_string()
            };
            targets.push(Target {
                from: name.clone(),
                to: name_only(&route.target.name),
            });
            rs.push_str(&format!(
                r#"
{name}
"#
            ));
        }
        rs.push_str(&format!(
            r#"end
"#
        ));
        s.push_str(&rs);
    }

    if events.len() > 0 {
        let mut es = format!("subgraph events");
        for (name, event) in events {
            es.push_str(&format!(
                r#"
{name}"#
            ));
            for target in &event.targets {
                targets.push(Target {
                    from: name.to_string(),
                    to: name_only(&target.name),
                });
            }
        }
        es.push_str(&format!(
            r#"
end
"#
        ));
        s.push_str(&es);
    }

    if functions.len() > 0 {
        let mut fs = format!("subgraph functions");
        for (name, function) in functions {
            fs.push_str(&format!(
                r#"
{name}"#
            ));
            for target in &function.targets {
                targets.push(Target {
                    from: name.to_string(),
                    to: name_only(&target.name),
                });
            }
        }
        fs.push_str(&format!(
            r#"
end
"#
        ));
        s.push_str(&fs);
    }

    if channels.len() > 0 {
        let mut cs = format!("subgraph channels");
        for (name, _) in channels {
            cs.push_str(&format!(
                r#"
{name}"#
            ));
        }
        cs.push_str(&format!(
            r#"
end
"#
        ));
        s.push_str(&cs);
    }

    if queues.len() > 0 {
        let mut qs = format!("subgraph queues");
        for (name, _) in queues {
            qs.push_str(&format!(
                r#"
{name}
"#
            ));
        }
        qs.push_str(&format!(
            r#"
end
"#
        ));

        s.push_str(&qs);
    }

    for target in &targets {
        let t = format!(
            r#"
{}-->{}
"#,
            target.from, target.to
        );
        s.push_str(&t);
    }

    if let Some(m) = mutations.get("default") {
        let mut ms = format!("subgraph mutations");
        for (name, res) in &m.resolvers {
            ms.push_str(&format!(
                r#"
{name}"#
            ));
            targets.push(Target {
                from: name.to_string(),
                to: res.target_name.to_string(),
            });
        }
        ms.push_str(&format!(
            r#"
end
"#
        ));
        s.push_str(&ms);
    }

    if let Some(_f) = flow {
        let mut ss = format!("subgraph states");
        ss.push_str(&format!(
            r#"
{namespace}
end
"#
        ));
        s.push_str(&ss);
    }

    let style = format!(
        r#"
    classDef red fill:#bfdbfe,color:#000,stroke:#333;
    classDef blue fill:#fcd34,color:#000,stroke:#333;
    classDef bing fill:#FFC3A0,color:#000,stroke:#333;
    classDef chan fill:#bbf7d0,color:#000,stroke:#333;
    class events bing
    class routes red
    class states blue
    class channels chan
"#
    );
    s.push_str(&style);
    s
}

pub fn build(topology: &Topology) -> String {
    let flow_str = build_flow(topology);
    let mermaid_str = format!(
        r#"
flowchart LR

{flow_str}
"#
    );
    mermaid_str
}

pub fn pprint(topology: &Topology) {
    let s = build(topology);
    println!("{}", &s);
}


pub fn pprint_recursive(_topologies: &HashMap<String, Topology>) {
}
