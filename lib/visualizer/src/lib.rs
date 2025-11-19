mod digraph;

use composer::Topology;
use kit as u;

use gv::{
    GraphBuilder,
    parser::DotParser,
};
use layout::{
    backends::svg::SVGWriter,
    gv,
};

pub fn render(name: &str, definition: &str, diagram_content: &str) -> String {
    format!(
        r#"

<!DOCTYPE html>
<html lang="en" data-theme="light">
  <head>
    <meta charset="UTF-8">
    <title>tc-{name}</title>
    <meta name="robots" content="noindex">
    <meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/default.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/languages/yaml.min.js"></script>
<script defer src="https://unpkg.com/@panzoom/panzoom@4.6.0/dist/panzoom.min.js"></script>
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
<body>

<div class="spaced">
<div class="x">
<div class="x15p">
	  <script>hljs.highlightAll();</script>
	  <pre><code class="language-yaml">
{definition}
	  </code></pre>

</div>


<div class="xg">
<div class="diagram-container spaced" id="diagram-container">
  <div class="spaced">{diagram_content}</div>
</div>
</div>
</div>
</div>

  </body>
</html>

"#
    )
}

pub fn generate_dot(topology: &Topology) -> String {
    let dot_str = digraph::build(topology);
    if dot_str.is_empty() {
        String::from("")
    } else {
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
}

pub fn generate(topology: &Topology) -> String {
    let definition = u::slurp(&format!("{}/topology.yml", &topology.dir));
    let dot_str = generate_dot(topology);
    render(&topology.namespace, &definition, &dot_str)
}

pub fn visualize(dir: &str) {
    let topology = composer::compose(dir, false);
    println!("Generating SVG...");
    let html = generate(&topology);
    let dir = u::pwd();
    let path = format!("{}/flow.html", &dir);
    u::write_str(&path, &html);
    println!("Opening {}", &path);
    open::that(path).unwrap();
}
