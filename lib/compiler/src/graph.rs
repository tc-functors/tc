use super::Topology;
use std::fmt;
use daggy::{Dag};
use serde_derive::Serialize;

#[derive(Debug, Clone, Serialize)]
enum NodeKind {
    Event,
    Function,
    Mutation,
    Channel,
    Route,
    Queue,
    StepFunction
}

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

fn make_node(name: &str, kind: &str) -> Node {
    Node {
        name: String::from(name),
        kind: String::from(kind)
    }
}


pub type Graph = Dag<String, Node>;

pub fn generate(t: &Topology) -> Graph {

    let Topology {
        namespace, .. } = t;

    let mut dag = Graph::new();

    let root = make_node(&namespace, "sfn");

    let n0 = dag.add_node("root".to_string());

    dag

}
