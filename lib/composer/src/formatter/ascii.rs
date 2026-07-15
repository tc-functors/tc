use crate::{
    Topology,
    graph,
};
use ascii_petgraph::RenderedGraph;

pub fn pprint(topology: &Topology) {
    let graph = graph::build_digraph(topology);

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
