use compiler::{
    Entity,
    FunctionSpec,
};
use serde_derive::{
    Deserialize,
    Serialize,
};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub name: String
}

impl Target {

    pub fn make_all(spec: &FunctionSpec) -> Vec<Target> {

        let mut xs: Vec<Target> = vec![];
        if let Some(tspecs) = &spec.targets {
            for tspec in tspecs {
                let t = Target {
                    entity: tspec.entity.clone(),
                    name: tspec.name.clone()
                };
                xs.push(t);
            }
        }
        xs
    }
}
