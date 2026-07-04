use crate::Topology;
use ascii_dag::Graph;

pub fn pprint(_topology: &Topology) {
    let mut g = Graph::new();
    g.add_node(1, "Web");
    g.add_node(2, "API");
    g.add_node(3, "DB");
    g.add_node(4, "Cache");

    g.add_edge(1, 2, None);
    g.add_edge(2, 3, None);
    g.add_edge(2, 4, None);

    // Create clusters
    let frontend = g.add_subgraph("Frontend");
    g.put_nodes(&[1]).inside(frontend).unwrap();

    let backend = g.add_subgraph("Backend");
    g.put_nodes(&[2, 3, 4]).inside(backend).unwrap();

    let ir = g.compute_layout();
    println!("{}", ir.render_scanline());
}
