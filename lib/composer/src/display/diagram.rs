use crate::Topology;
use kit as u;

pub fn html(definition: &str, data: &str) -> String {
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
<style>

      .spaced {{
	margin-left: 1rem;
	margin-right: 1rem;
      }}
      .x20p {{ flex: 0 0 20% }}
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

<script src="https://unpkg.com/mermaid@8.0.0/dist/mermaid.min.js"></script>

<div class="spaced" style="height: 94dvh;">
<div class="x">
<div class="x20p">
	  <script>hljs.highlightAll();</script>
	  <pre><code class="language-yaml">
{definition}
	  </code></pre>

</div>
<div class="xg">
<div class="diagram-container spaced" id="diagram-container">
  <div class="mermaid">
    {data}
  </div>
</div>
</div>
</div>
</div>
  </body>
</html>

"#)
}

pub fn generate(topology: &Topology) {
    let data = format!("
flowchart TB
    c1-->a2
    subgraph one
    a1-->a2
    end
    subgraph two
    b1-->b2
    end
    subgraph three
    c1-->c2
    end
    one --> two
    three --> two
    two --> c2
");
    let definition = u::slurp(&format!("{}/topology.yml", &topology.dir));
    let html = html(&definition, &data);
    let dir = u::pwd();
    let path = format!("{}/flow.html", &dir);
    u::write_str(&path, &html);
    open::that(path).unwrap();
}
