use crate::Topology;
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelConnection {
    pub id: String,
    pub direction: String,
    pub name: String,
    #[serde(rename(serialize = "OriginId"))]
    pub origin_id: Option<String>,
    #[serde(rename(serialize = "TargetId"))]
    pub target_id: Option<String>,
}

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
pub struct Tag {
    pub id: String,
    pub name: String,
    #[serde(rename(serialize = "GroupId"))]
    pub group_id: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Landscape {
    pub tags: Vec<Tag>,
    #[serde(rename(serialize = "tagGroups"))]
    pub tag_groups: Vec<ModelObject>,
    #[serde(rename(serialize = "modelConnections"))]
    pub model_connections: Vec<ModelConnection>,
    #[serde(rename(serialize = "modelObjects"))]
    pub model_objects: Vec<ModelObject>,
}

fn make_model_connections(_topologies: &HashMap<String, Topology>) -> Vec<ModelConnection> {
    vec![]
}

fn make_model_objects_node(domain: &str, name: &str, parent_id: Option<String>, topology: &Topology) -> Vec<ModelObject> {
    let mut mos: Vec<ModelObject> = vec![];

    let pmo = ModelObject {
        id: domain.to_string(),
        name: domain.to_string(),
        kind: "domain".to_string(),
        parent_id: parent_id,
        tag_ids: vec![],
    };
    mos.push(pmo);

    for (fname, _) in &topology.functions {
        let mo = ModelObject {
            id: format!("{}_{}", &name, &fname),
            name: fname.to_string(),
            parent_id: Some(name.to_string()),
            kind: "system".to_string(),
            tag_ids: vec![],
        };
        mos.push(mo);
    }

    for (ename, _) in &topology.events {
        let mo = ModelObject {
            id: format!("{}_{}", &name, &ename),
            name: ename.to_string(),
            parent_id: Some(name.to_string()),
            kind: "system".to_string(),
            tag_ids: vec![],
        };
        mos.push(mo);
    }
    mos
}

fn make_model_objects(topologies: &HashMap<String, Topology>) -> Vec<ModelObject> {
    let mut mos: Vec<ModelObject> = vec![];

    for (name, topology) in topologies {

        let objects = make_model_objects_node(name, name, None, topology);

        mos.extend(objects);

        for (n, node) in &topology.nodes {

            let node_objects = make_model_objects_node(n, name, None, node);
            mos.extend(node_objects);
        }
    }
    mos
}

fn make_tags() -> Vec<Tag> {
    vec![]
}


fn build(topologies: &HashMap<String, Topology>) -> Landscape {
    Landscape {
        model_connections: make_model_connections(topologies),
        model_objects: make_model_objects(topologies),
        tags: make_tags(),
        tag_groups: vec![]
    }
}

pub fn pprint(topology: &Topology) {
    let mut t: HashMap<String, Topology> = HashMap::new();
    t.insert(topology.namespace.clone(), topology.clone());
    let objs = build(&t);
    u::pp_json(&objs);
}

pub fn pprint_recursive(topologies: &HashMap<String, Topology>) {
    let objs = build(topologies);
    u::pp_json(&objs);
}
