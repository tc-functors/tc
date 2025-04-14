use super::Topology;
use std::fmt;
use daggy::{Dag};
use serde_derive::Serialize;


#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub name: String,
    pub kind: String,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.name)
    }
}


pub type Graph = Dag<String, Node>;

pub fn generate(_t: &Topology) -> Graph {

    // let Topology {
    //     namespace, .. } = t;

    let dag = Graph::new();

    //let root = make_node(&namespace, "sfn");

    //let n0 = dag.add_node("root".to_string());

    dag

}
