mod html;
mod graph;
use composer::Topology;
use kit as u;
use layout::gv;
use gv::parser::DotParser;
use gv::GraphBuilder;
use layout::backends::svg::SVGWriter;

pub fn generate_dot(topology: &Topology) -> String {
    let dot_str = graph::build(topology);
    let mut parser = DotParser::new(&dot_str);

    let tree = parser.process();

    match tree {
        Result::Err(err) => {
            parser.print_error();
            println!("Error: {}", err);
            String::from("")
        }

        Result::Ok(g) => {
            gv::dump_ast(&g);

            let mut gb = GraphBuilder::new();
            gb.visit_graph(&g);
            let mut vg = gb.get();
            let mut svg = SVGWriter::new();
            vg.do_it(false, false, false, &mut svg);
            svg.finalize()
        }
    }
}

pub fn generate_aws_arch(_topology: &Topology) -> String {
    format!(r#"
architecture-beta
    group api(logos:aws-lambda)[API]

    service db(logos:aws-aurora)[Database] in api
    service disk1(logos:aws-glacier)[Storage] in api
    service disk2(logos:aws-s3)[Storage] in api
    service server(logos:aws-ec2)[Server] in api

    db:L -- R:server
    disk1:T -- B:server
    disk2:T -- B:db
"#)
}

struct Target {
    from: String,
    to: String
}

fn name_only(s: &str) -> String {
    if s.starts_with("{{") {
        u::second(&s, "_")
    } else {
        s.to_string()
    }
}

fn generate_mermaid(topology: &Topology) -> String {
    let mut targets: Vec<Target> = vec![];
    let mut s: String =  String::from("");

    let Topology { routes, events, channels, mutations,
                   functions, queues, namespace, flow, .. } = topology;

    if routes.len() > 0 {
        let mut rs = format!("subgraph routes");
        for (name, route) in routes {
            let name = if name.starts_with("/") {
                format!("route_{}{{{}}}",
                        &u::split_last(name, "/"),
                        route.path
                )
            } else {
                name.to_string()
            };
            targets.push(Target {
                from: name.clone(), to: name_only(&route.target.name)
            });
            rs.push_str(&format!(r#"
{name}
"#));
        }
        rs.push_str(&format!(r#"end
"#));
        s.push_str(&rs);
    }

    if events.len() > 0 {
        let mut es = format!("subgraph events");
        for (name, event) in events {
     es.push_str(&format!(r#"
{name}"#));
            for target in &event.targets {
                targets.push(Target {
                    from: name.to_string(),
                    to: name_only(&target.name)
                });
            }
        }
        es.push_str(&format!(r#"
end
"#));
        s.push_str(&es);
    }

    if functions.len() > 0 {
        let mut fs = format!("subgraph functions");
        for (name, function) in functions {
     fs.push_str(&format!(r#"
{name}"#));
            for target in &function.targets {
                targets.push(Target {
                    from: name.to_string(),
                    to: name_only(&target.name)
                });
            }
        }
        fs.push_str(&format!(r#"
end
"#));
        s.push_str(&fs);
    }

    if channels.len() > 0 {
        let mut cs = format!("subgraph channels");
        for (name, _) in channels {
     cs.push_str(&format!(r#"
{name}"#));
        }
        cs.push_str(&format!(r#"
end
"#));
        s.push_str(&cs);
    }

    if queues.len() > 0 {
        let mut qs = format!("subgraph queues");
        for (name, _) in queues {
     qs.push_str(&format!(r#"
{name}
"#));
        }
        qs.push_str(&format!(r#"
end
"#));

        s.push_str(&qs);
    }

    for target in &targets {
        let t = format!(r#"
{}-->{}
"#, target.from, target.to);
        s.push_str(&t);
    }

    if let Some(m) = mutations.get("default") {
        let mut ms = format!("subgraph mutations");
        for (name, res) in &m.resolvers {
     ms.push_str(&format!(r#"
{name}"#));
            targets.push(Target {
                from: name.to_string(),
                to: res.target_name.to_string()
            });
        }
        ms.push_str(&format!(r#"
end
"#));
        s.push_str(&ms);
    }

    if let Some(_f) = flow {
        let mut ss = format!("subgraph states");
        ss.push_str(&format!(r#"
{namespace}
end
"#));
        s.push_str(&ss);
    }

    let style = format!(r#"
    classDef red fill:#ffefdf,color:#000,stroke:#333;
    classDef blue fill:#e4fbfc,color:#000,stroke:#333;
    classDef bing fill:#f1edff,color:#000,stroke:#333;
    classDef chan fill:#deffe5,color:#000,stroke:#333;
    class events blue
    class routes red
    class states bing
    class channels chan
"#
);
    s.push_str(&style);

    s
}

pub fn visualize(topology: &Topology) {
    println!("Generating SVG...");
    let flow_str = generate_mermaid(topology);
    let mermaid_str = format!(r#"
flowchart LR

{flow_str}
"#);
    let dot_str = generate_dot(topology);
    let aws_str = generate_aws_arch(topology);
    let definition = u::slurp(&format!("{}/topology.yml", &topology.dir));
    let html = html::generate(
        &topology.namespace,
        &definition,
        &mermaid_str,
        &dot_str,
        &aws_str,
        &topology.to_str()
    );
    let dir = u::pwd();
    let path = format!("{}/flow.html", &dir);
    u::write_str(&path, &html);
    println!("Opening {}", &path);
    open::that(path).unwrap();
}
