mod spec;
mod yaml;

pub use spec::{
    TopologyKind,
    TopologySpec,
    function::{
        BuildKind,
        Lang,
        LangRuntime,
    },
    infra::InfraSpec,
};

pub fn compile(dir: &str, _recursive: bool) -> TopologySpec {
    let file = format!("{}/topology.yml", dir);
    TopologySpec::new(&file)
}
