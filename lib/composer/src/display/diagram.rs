use crate::Topology;
use kit as u;

pub fn html(definition: &str, data: &str, topology_str: &str) -> String {
    format!(r#"

<!DOCTYPE html>
<html lang="en" data-theme="light">
  <head>
    <meta charset="UTF-8">
    <title>tc</title>
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/default.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/yaml.min.js"></script>
<script defer src="https://unpkg.com/@panzoom/panzoom@4.6.0/dist/panzoom.min.js"></script>
<script src="https://unpkg.com/alpinejs@3.10.5/dist/cdn.min.js" defer></script>
<script src="https://unpkg.com/@alenaksu/json-viewer@2.1.0/dist/json-viewer.bundle.js"></script>
<style>

json-viewer {{
    /* Background, font and indentation */
    --background-color: #ffffff;
    --color: #000000;
     --font-family: Nimbus Mono PS, Courier New, monospace;
    --font-size: 0.9rem;
    --line-height: 1.2rem;
    --indent-size: 0.9em;
    --indentguide-size: 0px;
    --string-color: green;
    --property-color: blue;
}}

.mermaid svg {{
    display: block;
    width: 100%;
    margin: 0;
    padding: 0;
}}

/** On hover, make the diagram full width and enable horizontal scrolling */

div:has(> .mermaid):hover {{
    width: auto !important;
}}

.mermaid:hover {{
    overflow: scroll;
    padding: 0;
    margin: 0;
    text-align: left;
}}

.mermaid:hover svg {{
    display: block;
    width: auto;
    margin: 0;
    padding: 0;
}}

  ul.tabs {{
          display: table;
          list-style-type: none;
          margin: 0;
          padding: 0;
      }}

      ul.tabs>li {{
          float: left;
          padding: 10px;
      }}

      ul.tabs>li:hover {{
          background-color: lightgray;
      }}

      ul.tabs>li.selected {{
          background-color: lightgray;
      }}

      div.content {{
          border: 1px solid black; overflow: auto;

      }}

      ul {{ overflow: auto; }}

      div.content {{ clear: both;  height: 94vh; }}
      .spaced {{
	margin-left: 1rem;
	margin-right: 1rem;
      }}
      .x10p {{ flex: 0 0 10% }}
      .xg {{ flex: 1 0 auto }}
      @media (min-width: 992px) {{ .x {{ display: flex; }} .x > * + * {{margin-left: 0rem}}}}
</style>
</head>
<body>
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
  mermaid.initialize({{
  startOnLoad: true,
  theme: 'default',
  sequence: {{ showSequenceNumbers: true }} }});
  await mermaid.run({{
  querySelector: '.mermaid',
  postRenderCallback: (id) => {{
  const container = document.getElementById("diagram-container");
  const svgElement = container.querySelector("svg");

  // Initialize Panzoom
  const panzoomInstance = Panzoom(svgElement, {{
  maxScale: 5,
  minScale: 0.5,
  step: 0.1,
  }});

  // Add mouse wheel zoom
  container.addEventListener("wheel", (event) => {{
  panzoomInstance.zoomWithWheel(event);
  }});
  }}
  }});
</script>


<script>
function init_tabs() {{
  return {{
    currentTab: 1,
    goFullScreen() {{

        var elem = document.getElementById('diagram-container');

        if(elem.requestFullscreen){{
            elem.requestFullscreen();
        }}
        else if(elem.mozRequestFullScreen){{
            elem.mozRequestFullScreen();
        }}
        else if(elem.webkitRequestFullscreen){{
            elem.webkitRequestFullscreen();
        }}
        else if(elem.msRequestFullscreen){{
            elem.msRequestFullscreen();
        }}
    }}
}}
}}
</script>

<script src="https://unpkg.com/mermaid@8.0.0/dist/mermaid.min.js"></script>

<div class="spaced">
<div class="x">
<div class="x15p">
	  <script>hljs.highlightAll();</script>
	  <pre><code class="language-yaml">
{definition}
	  </code></pre>

</div>
<div class="xg">
<div class="spaced">

<div x-data="init_tabs()">
    <div align="left">
	<ul class="tabs">
		<li @click="currentTab = 1">
			<button href="r#">Flow Diagram</button>
		</li>
		<li @click="currentTab = 2">
			<button href="r#">JSON</a>
		</li>
		<li @click="currentTab = 3">
			<button href="r#">Table</a>
		</li>

	</ul>
   </div>
	<div class="content">
		<div x-show="currentTab === 1">

<div align="right">
<button @click="goFullScreen()">fullscreen</button>
</div>
<div class="diagram-container spaced" id="diagram-container">
  <div class="mermaid">
    {data}
  </div>
</div>
                </div>
		<div x-show="currentTab === 2">
<div class="spaced">
<json-viewer id="json"></json-viewer>
<script>
    document.querySelector('#json').data = {topology_str};
</script>
</div>
</div>
		<div x-show="currentTab === 3">
Tables
</div>


	</div>
</div>
</div>

</div>
</div>
</div>
  </body>
</html>

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

fn build(topology: &Topology) -> String {
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

pub fn generate(topology: &Topology) {
    println!("Generating SVG...");
    let flow_str = build(topology);
    let mermaid_str = format!(r#"
flowchart LR
{flow_str}
"#);
    let definition = u::slurp(&format!("{}/topology.yml", &topology.dir));
    let html = html(&definition, &mermaid_str, &topology.to_str());
    let dir = u::pwd();
    let path = format!("{}/flow.html", &dir);
    u::write_str(&path, &html);
    println!("Opening {}", &path);
    open::that(path).unwrap();
}
