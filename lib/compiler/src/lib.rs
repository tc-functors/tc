mod yaml;
pub mod spec;
pub mod entity;

pub use spec::{
    TopologyKind,
    TopologySpec,
    function::{
        BuildKind,
        Lang,
        LangRuntime,
        FunctionSpec
    },
    infra::InfraSpec,
};
pub use entity::Entity;

pub fn compile(dir: &str, _recursive: bool) -> TopologySpec {
    let file = format!("{}/topology.yml", dir);
    TopologySpec::new(&file)
}
