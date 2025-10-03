use compiler::spec::function::{
    build::BuildKind,
    runtime::LangRuntime,
};

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub name: String,
    pub dir: String,
    pub runtime: LangRuntime,
    pub kind: BuildKind,
    pub artifact: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildStatus {
    pub path: String,
    pub status: bool,
    pub out: String,
    pub err: String,
}
