use composer::Topology;
use kit as u;
use rand::Rng;
use std::collections::{
    HashMap,
    hash_map::Entry,
};

pub fn render_dark(mermaid_str: &str) -> String {
    format!(
        r#"

<!DOCTYPE html>
<html lang="en" data-theme="dark">
  <head>
    <meta charset="UTF-8">
    <title>tc</title>
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
<script defer src="https://unpkg.com/@panzoom/panzoom@4.6.0/dist/panzoom.min.js"></script>
<script src="https://unpkg.com/alpinejs@3.10.5/dist/cdn.min.js" defer></script>
<style>

body {{
  background: #000;
  color: #fff;
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

      div.content {{ clear: both;  height: 90vh; }}
      .spaced {{
	margin-left: 1rem;
	margin-right: 1rem;
      }}
      .x10p {{ flex: 0 0 10% }}
      .xg {{ flex: 1 0 auto }}
      @media (min-width: 992px) {{ .x {{ display: flex; }} .x > * + * {{margin-left: 0rem}}}}
</style>
</head>
<body">
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11.12.0/dist/mermaid.esm.min.mjs';
  mermaid.initialize({{
  startOnLoad: true,
  theme: 'dark',
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

<div class="spaced content" x-data="init_tabs()">
  <div align="right">
    <button @click="goFullScreen()">fullscreen</button>
  </div>
<div class="diagram-container spaced" id="diagram-container">
  <div class="mermaid">
    {mermaid_str}
  </div>
</div>

</div>

  </body>
</html>

"#
    )
}

pub fn render(mermaid_str: &str) -> String {
    format!(
        r#"

<!DOCTYPE html>
<html lang="en" data-theme="light">
  <head>
    <meta charset="UTF-8">
    <title>tc</title>
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
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

      div.content {{ clear: both;  height: 90vh; }}
      .spaced {{
	margin-left: 1rem;
	margin-right: 1rem;
      }}
      .x10p {{ flex: 0 0 10% }}
      .xg {{ flex: 1 0 auto }}
      @media (min-width: 992px) {{ .x {{ display: flex; }} .x > * + * {{margin-left: 0rem}}}}
</style>
</head>
<body">
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11.12.0/dist/mermaid.esm.min.mjs';
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

<script src="https://unpkg.com/mermaid@8.0.0/dist/mermaid.min.js"></script>

<div class="spaced content">
  <div align="right">
    <button @click="goFullScreen()">fullscreen</button>
  </div>
<div class="diagram-container spaced" id="diagram-container">
  <div class="mermaid">
    {mermaid_str}
  </div>
</div>

</div>

  </body>
</html>

"#
    )
}

fn name_only(s: &str) -> String {
    if s.starts_with("{{") {
        u::second(&s, "_")
    } else if s.ends_with("}}") {
        u::first(&s, "_")
    } else {
        s.to_string()
    }
}

fn group_targets(topologies: &HashMap<String, Topology>) -> HashMap<String, Vec<String>> {
    let mut h: HashMap<String, Vec<String>> = HashMap::new();
    for (name, topology) in topologies {
        for (ename, event) in &topology.events {
            for target in &event.targets {
                let gname = name_only(&target.name);
                let tname = format!(
                    "{},{},{}_{}[{}]",
                    &target.producer_ns,
                    ename,
                    target.entity.to_str(),
                    &gname,
                    &gname
                );

                match h.entry(name.to_string()) {
                    Entry::Vacant(e) => {
                        e.insert(vec![tname]);
                    }
                    Entry::Occupied(mut e) => {
                        if !e.get_mut().contains(&tname) {
                            e.get_mut().push(tname);
                        }
                    }
                }
            }
        }
    }
    h
}

pub fn generate_mermaid(topologies: &HashMap<String, Topology>, theme: &str) -> String {
    let mut s: String = String::from("");

    let grouped = group_targets(topologies);

    for (name, targets) in &grouped {
        let begin = format!(
            r#"
subgraph {name}
"#
        );
        s.push_str(&begin);
        let end = format!(
            r#"
end
"#
        );
        for target in targets {
            let tname = u::split_last(&target, ",");
            let f = format!(
                r#"{tname}
"#
            );
            s.push_str(&f);
        }
        s.push_str(&end);
    }
    for (_, targets) in &grouped {
        for target in targets {
            let parts: Vec<&str> = target.split(",").collect();
            let producer = parts.clone().into_iter().nth(0).unwrap();
            let event = parts.clone().into_iter().nth(1).unwrap();
            let tname = parts.clone().into_iter().nth(2).unwrap();
            if producer != "sandbox" {
                let f = format!(
                    r#"
{producer}--{event}-->{tname}
"#
                );
                s.push_str(&f);
            }
        }
    }

    if theme != "dark" {
        let mut style = format!(
            r#"
    classDef red fill:#ffefdf,color:#000,stroke:#333;
    classDef blue fill:#e4fbfc,color:#000,stroke:#333;
    classDef bing fill:#f1edff,color:#000,stroke:#333;
    classDef chan fill:#deffe5,color:#000,stroke:#333;
    classDef c1 fill:#DE8F5F,color:#000,stroke:#333;
    classDef c2 fill:#FFB26F,color:#000,stroke:#333;
    classDef c3 fill:#F1C27B,color:#000,stroke:#333;
    classDef c4 fill:#FFD966,color:#000,stroke:#333;
"#
        );
        let strings = vec!["red", "blue", "bing", "chan", "c1", "c2", "c3", "c4"];
        for (name, _) in grouped {
            let random_class = &strings[rand::rng().random_range(0..strings.len())];
            let p = format!(
                r#"
class {name} {random_class}
"#
            );
            style.push_str(&p);
        }
        s.push_str(&style);
    }
    s
}

pub fn generate(topologies: &HashMap<String, Topology>, theme: &str) -> String {
    let flow_str = generate_mermaid(topologies, theme);
    let mermaid_str = format!(
        r#"
flowchart LR
{flow_str}
"#
    );
    if theme == "dark" {
        render_dark(&mermaid_str)
    } else {
        render(&mermaid_str)
    }
}
