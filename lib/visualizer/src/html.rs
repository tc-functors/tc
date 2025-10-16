
pub fn generate(name: &str, definition: &str, mermaid_str: &str, dot_str: &str, aws_str: &str, topology_str: &str) -> String {
    format!(r#"

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
<body>
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11.12.0/dist/mermaid.esm.min.mjs';
  mermaid.initialize({{
  startOnLoad: true,
  theme: 'default',
  sequence: {{ showSequenceNumbers: true }} }});

mermaid.registerIconPacks([
  {{
    name: 'logos',
    loader: () =>
      fetch('https://unpkg.com/@iconify-json/logos@1/icons.json').then((res) => res.json()),
  }},
]);
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
			<button href="r#">Mermaid</button>
		</li>
		<li @click="currentTab = 2">
			<button href="r#">Dot</button>
		</li>
		<li @click="currentTab = 3">
			<button href="r#">JSON</a>
		</li>
		<li @click="currentTab = 4">
			<button href="r#">AWS Arch</a>
		</li>
		<li @click="currentTab = 5">
			<button href="r#">S-exp</a>
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
    {mermaid_str}
  </div>
</div>
                </div>

		<div x-show="currentTab === 2">
<div class="spaced">
{dot_str}
</div>
</div>


		<div x-show="currentTab === 3">
<div class="spaced">
<json-viewer id="json"></json-viewer>
<script>
    document.querySelector('#json').data = {topology_str};
</script>
</div>
</div>
		<div x-show="currentTab === 4">
<div align="center">
<br/>
</br>
<div class="diagram-container spaced" id="diagram-container">
  <div class="mermaid">
    {aws_str}
  </div>
</div>
</div>

		<div x-show="currentTab === 5">
<pre><code>
(compose
  (route "/api/todo" :method 'POST)
  (event 'MyEvent)
  (table 'MyTable))

(compose
  (table 'MyTable :listen {{:key 'foo}})
  (channel my-channel))
</code></pre>
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
