use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

use crate::Topology;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelObject {
    pub id: String,
    pub name: String,
    #[serde(rename(serialize = "parentId"))]
    pub parent_id: String,
    #[serde(rename(serialize = "type"))]
    pub kind: String,
    #[serde(rename(serialize = "tagIds"))]
    pub tag_ids: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Landscape {
    #[serde(rename(serialize = "modelObjects"))]
    pub model_objects: Vec<ModelObject>
}

pub fn generate(topologies: &HashMap<String, Topology>) -> Landscape {
    let mut mos: Vec<ModelObject> = vec![];
    for (name, topology) in topologies {
        let pmo = ModelObject {
            id: name.to_string(),
            name: name.to_string(),
            parent_id: name.to_string(),
            kind: "domain".to_string(),
            tag_ids: vec!["tag-external".to_string()]
        };
        mos.push(pmo);
        for (n, _) in &topology.nodes {
            let mo = ModelObject {
                id: n.to_string(),
                name: n.to_string(),
                parent_id: name.to_string(),
                kind: "domain".to_string(),
                tag_ids: vec!["tag-external".to_string()]
            };
            mos.push(mo);
        }
    }
    Landscape {
        model_objects: mos
    }
}
