mod aws;
mod graph;
mod printer;
mod diagram;

pub mod topology;
use compiler::{Entity, TopologySpec};
pub use aws::function::{
    Function,
    Build,
    runtime::Runtime

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
use printer::Format;
use std::str::FromStr;
use kit as u;

pub fn compose(spec: &TopologySpec) -> Topology {
    Topology::new(spec)
}

pub fn pprint(topology: &Topology, fmt: &str) {
    let format = Format::from_str(fmt).unwrap();
    match format {
        Format::Dot => printer::print_dot(topology),
        Format::JSON => u::pp_json(topology),
        _ => ()
    }
}

pub fn generate_diagram(topology: &Topology, kind: &str) {
    match kind {
        "sequence" => diagram::render_sequence(topology),
        _ => ()
    }
}


pub fn print_entity(topology: &Topology, e: &str, f: &str) {
    let format = Format::from_str(f).unwrap();
    match e {
        "." => {
            let dir = u::pwd();
            if let Some(f) = topology.current_function(&dir) {
                u::pp_json(&f)
            }
        }
        "roles" => {
            u::pp_json(&topology.roles);
        }
        _ => {
            let entity = Entity::from_str(e).unwrap();
            printer::print_entity(&topology, entity, format)
        }
    }
}

pub fn entities_of(topology: &Topology) -> Vec<Entity> {
    let Topology {
        routes,
        events,
        channels,
        queues,
        functions,
        pages,
        mutations,
        flow,
        ..
    } = topology;
    let mut xs: Vec<Entity> = vec![];

    if functions.len() > 0 {
        xs.push(Entity::Function)
    }
    if routes.len() > 0 {
        xs.push(Entity::Route)
    }
    if events.len() > 0 {
        xs.push(Entity::Event)
    }
    if pages.len() > 0 {
        xs.push(Entity::Page)
    }
    if channels.len() > 0 {
        xs.push(Entity::Channel)
    }
    if queues.len() > 0 {
        xs.push(Entity::Queue)
    }
    if let Some(_f) = flow {
        xs.push(Entity::State);
    }
    if mutations.len() > 0 {
        if let Some(m) = mutations.get("default") {
            if m.resolvers.len() > 0 {
                xs.push(Entity::Mutation)
            }
        }
    }

    xs
}

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

pub fn count_of(spec: &Topology) -> String {
    printer::count_str(spec)
}
