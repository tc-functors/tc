use composer::Topology;
use daggy::Dag;
use daggy::petgraph::dot::{Dot, Config};

//use serde_derive::Serialize;
//use std::fmt;

//pub type Graph = Dag<String, Node>;

pub fn build(_t: &Topology) -> String {
    let mut dag = Dag::<&str, &str>::new();
    let a = dag.add_node("a");

    let (_, b) = dag.add_child(a, "a->b", "b");
    let (_, c) = dag.add_child(a, "a->c", "c");
    let (_, d) = dag.add_child(a, "a->d", "d");
    let (_, e) = dag.add_child(a, "a->e", "e");

    dag.add_edge(b, d, "b->d").unwrap();

    dag.add_edge(c, d, "c->d").unwrap();
    dag.add_edge(c, e, "c->e").unwrap();

    dag.add_edge(d, e, "d->e").unwrap();

    dag.transitive_reduce(vec![a]);

    format!("{}", Dot::with_config(&dag, &[Config::EdgeNoLabel]))
}


// pub fn digraph(dag: &Graph) -> String {
//     format!("{}", Dot::with_config(dag, &[Config::EdgeNoLabel]))
// }
