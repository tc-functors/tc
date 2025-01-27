use serde_derive::Serialize;
use compiler::Topology;
use compiler::spec::{LangRuntime};
use kit as u;

#[derive(Debug, Clone, Serialize)]
pub struct Layer {
    pub name: String,
    pub layers: Vec<String>,
    pub lang: LangRuntime,
}

pub fn render_layers(topologies: &Vec<Topology>) -> String {
    let mut layers: Vec<Layer> = vec![];
    for t in topologies {
        let functions = &t.functions;
        for (_, f) in functions {
            if f.runtime.layers.len() > 0 {
                let layer = Layer {
                    name: f.name.to_owned(),
                    layers: f.runtime.layers.to_owned(),
                    lang: f.runtime.lang.to_owned(),
                };
                layers.push(layer)
            }
        }
    }
    u::pretty_json(&layers)
}
