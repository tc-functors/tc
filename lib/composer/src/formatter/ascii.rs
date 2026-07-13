use crate::Topology;
use ascii_petgraph::RenderedGraph;
use petgraph::graph::DiGraph;

pub fn pprint(_topology: &Topology) {
    let mut graph = DiGraph::new();
    let a = graph.add_node("Start");
    let b = graph.add_node("Process");
    let c = graph.add_node("End");

    graph.add_edge(a, b, "begin");
    graph.add_edge(b, c, "finish");

    // Render it
    let mut rendered = RenderedGraph::from_graph(graph);
    rendered.run_simulation();

    // Print to terminal
    let grid = rendered.render_to_grid();
    for y in 0..grid.size().1 {
        for x in 0..grid.size().0 {
            if let Some(cell) = grid.get(x, y) {
                print!("{}", cell.char);
            }
        }
        println!();
    }
}
