mod aws;
mod graph;

pub mod topology;
pub use aws::function::{
    Function,
    Build
};
pub use aws::event::{Event, Target};
pub use aws::channel::Channel;
pub use aws::mutation::Mutation;
pub use aws::page::{Page, BucketPolicy};
pub use aws::queue::Queue;
pub use aws::role::Role;
pub use aws::route::Route;
pub use aws::schedule::Schedule;
pub use aws::flow::Flow;

pub use aws::function;
pub use aws::page;
pub use topology::Topology;


pub fn compose(dir: &str, recursive: bool) -> Topology {
    let spec = compiler::compile(dir, recursive);
    Topology::new(&spec)
}

// deprecated
pub fn find_buildables(dir: &str, recursive: bool) -> Vec<Build> {
    let mut xs: Vec<Build> = vec![];
    let spec = compiler::compile(dir, recursive);
    let topology = Topology::new(&spec);
    let fns = topology.functions;
    for (_, f) in fns {
        xs.push(f.build)
    }
    xs
}

pub fn current_function(dir: &str) -> Option<Function> {
    let spec = compiler::compile(dir, false);
    let topology = Topology::new(&spec);
    topology.current_function(dir)
}
