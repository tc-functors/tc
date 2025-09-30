use serde_derive::{
    Deserialize,
    Serialize,
};
use kit::*;
use kit as u;
use std::{
    str::FromStr,
};

use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

fn default_command() -> String {
    s!("zip -9 -r lambda.zip .")
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BuildKind {
    #[serde(alias = "code")]
    Code,
    #[serde(alias = "inline")]
    Inline,
    #[serde(alias = "layer")]
    Layer,
    #[serde(alias = "slab")]
    Slab,
    #[serde(alias = "library")]
    Library,
    #[serde(alias = "extension")]
    Extension,
    #[serde(alias = "runtime")]
    Runtime,
    #[serde(alias = "image")]
    Image,
}

impl FromStr for BuildKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Code" | "code" => Ok(BuildKind::Code),
            "Inline" | "inline" => Ok(BuildKind::Inline),
            "layer" => Ok(BuildKind::Layer),
            "library" => Ok(BuildKind::Library),
            "extension" => Ok(BuildKind::Extension),
            "runtime" => Ok(BuildKind::Runtime),
            "slab" => Ok(BuildKind::Slab),
            "Image" | "image" => Ok(BuildKind::Image),
            _ => Ok(BuildKind::Code),
        }
    }
}

impl BuildKind {
    pub fn to_str(&self) -> String {
        match self {
            BuildKind::Code => s!("code"),
            BuildKind::Inline => s!("inline"),
            BuildKind::Layer => s!("layer"),
            BuildKind::Library => s!("library"),
            BuildKind::Extension => s!("extension"),
            BuildKind::Runtime => s!("runtime"),
            BuildKind::Image => s!("image"),
            BuildKind::Slab => s!("slab"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildSpec {
    // deprecated
    pub kind: BuildKind,

    #[serde(default)]
    pub pre: Vec<String>,

    #[serde(default)]
    pub post: Vec<String>,

    #[serde(default)]
    pub package_manager: Option<String>,

    #[serde(default)]
    pub shared_context: Option<bool>,

    #[serde(default)]
    pub skip_dev_deps: Option<bool>,

    /// Command to use when build kind is Code
    #[serde(default = "default_command")]
    pub command: String,

    pub version: Option<String>,
}


fn infer_kind(package_type: &str) -> BuildKind {
    match package_type {
        "zip" => BuildKind::Code,
        "image" | "oci" => BuildKind::Image,
        "library" => BuildKind::Library,
        "extension" => BuildKind::Library,
        "zip-layer" | "layer" => BuildKind::Layer,
        "zip-inline" | "inline" => BuildKind::Inline,
        _ => BuildKind::Code,
    }
}

impl BuildSpec {
    pub fn new(dir: &str) -> BuildSpec {
        let path = format!("{}/build.json", dir);
        let data = u::slurp(&path);
        let bspec: BuildSpec = serde_json::from_str(&data).unwrap();
        bspec
    }

    pub fn augment(&self, package_type: &str) -> BuildSpec {
        let kind = infer_kind(package_type);
        let mut bs = self.clone();
        bs.kind = kind;
        bs
    }

    pub fn default(tasks: &HashMap<String, String>) -> Self {
        let command = match tasks.get("build") {
            Some(c) => c.to_owned(),
            None => s!("zip -9 -q lambda.zip *.*"),
        };
        BuildSpec {
            kind: BuildKind::Code,
            pre: vec![],
            post: vec![],
            version: None,
            command: command,
            package_manager: None,
            shared_context: Some(false),
            skip_dev_deps: Some(false),
        }
    }

}
