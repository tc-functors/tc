use serde_derive::{Deserialize, Serialize};

use kit::*;
use crate::spec::{Kind, BuildSpec};
use super::Runtime;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Build {
    pub kind: Kind,
    pub pre: Vec<String>,
    pub post: Vec<String>,
    pub command: String
}

fn infer_kind(package_type: &str) -> Kind {
    match package_type {
        "zip"                   => Kind::Code,
        "image" | "oci"         => Kind::Image,
        "library"               => Kind::Library,
        "extension"             => Kind::Library,
        "zip-layer" | "layer"   => Kind::Layer,
        "zip-inline" | "inline" => Kind::Inline,
        _                       => Kind::Code
    }
}

impl Build {



    pub fn new(runtime: &Runtime, bspec: Option<BuildSpec>) -> Build {
        match bspec {
            Some(b) => Build {
                kind: b.kind,
                pre: b.pre,
                post: b.post,
                command: b.command
            },
            None => {
                Build {
                    kind: infer_kind(&runtime.package_type),
                    pre: vec![],
                    post: vec![],
                    command: s!("zip -9 -q lambda.zip .")
                }
            }
        }
    }
}
