use composer::spec::{
    LangRuntime,
    function::BuildKind,
};

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub name: String,
    pub dir: String,
    pub runtime: LangRuntime,
    pub kind: BuildKind,
    pub artifact: String,
}

#[derive(Debug, Clone)]
pub struct BuildStatus {
    pub path: String,
    pub status: bool,
    pub out: String,
    pub err: String,
}
