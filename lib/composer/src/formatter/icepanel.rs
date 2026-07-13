use crate::Topology;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use kit as u;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelObject {
    pub id: String,
    pub name: String,
    #[serde(
        rename(serialize = "parentId"),
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_id: Option<String>,
    #[serde(rename(serialize = "type"))]
    pub kind: String,
    #[serde(rename(serialize = "tagIds"))]
    pub tag_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Landscape {
    #[serde(rename(serialize = "modelObjects"))]
    pub model_objects: Vec<ModelObject>,
}

fn generate(topologies: &HashMap<String, Topology>) -> Landscape {
    let mut mos: Vec<ModelObject> = vec![];
    for (name, topology) in topologies {
        let pmo = ModelObject {
            id: name.to_string(),
            name: name.to_string(),
            kind: "domain".to_string(),
            parent_id: None,
            tag_ids: vec!["tag-external".to_string()],
        };
        mos.push(pmo);
        for (fname, _) in &topology.functions {
            let mo = ModelObject {
                id: format!("{}_{}", &name, &fname),
                name: fname.to_string(),
                parent_id: Some(name.to_string()),
                kind: "app".to_string(),
                tag_ids: vec!["tag-external".to_string()],
            };
            mos.push(mo);
        }
        for (n, node) in &topology.nodes {
            let mo = ModelObject {
                id: n.to_string(),
                name: n.to_string(),
                parent_id: Some(name.to_string()),
                kind: "system".to_string(),
                tag_ids: vec!["tag-external".to_string()],
            };
            mos.push(mo);
            for (fname, _) in &node.functions {
                let mo = ModelObject {
                    id: format!("{}_{}", &n, &fname),
                    name: fname.to_string(),
                    parent_id: Some(n.to_string()),
                    kind: "app".to_string(),
                    tag_ids: vec!["tag-external".to_string()],
                };
                mos.push(mo);
            }
        }
    }
    Landscape { model_objects: mos }
}

pub fn pprint(topology: &Topology) {
    let mut t: HashMap<String, Topology> = HashMap::new();
    t.insert(topology.namespace.clone(), topology.clone());
    let objs = generate(&t);
    u::pp_json(&objs);
}

pub fn pprint_recursive(topologies: &HashMap<String, Topology>) {
    let objs = generate(topologies);
    u::pp_json(&objs);
}
