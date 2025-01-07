use super::{Context, Topology};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    pub name: String,
    pub consumer: String,
    pub producer: String,
}

pub async fn make(context: &Context, topology: &Topology) -> Vec<Queue> {
    let mut xs: Vec<Queue> = vec![];
    let Topology { queues, .. } = topology;
    for (k, q) in queues {
        let queue = Queue {
            name: context.render(&k),
            consumer: context.render(&q.consumer),
            producer: context.render(&q.producer),
        };
        xs.push(queue)
    }
    xs
}
