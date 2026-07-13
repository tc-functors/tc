use super::Runtime;
use compiler::spec::function::{
    BuildKind,
    BuildSpec,
};
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Build {
    pub dir: String,
    pub kind: BuildKind,
    pub pre: Vec<String>,
    pub post: Vec<String>,
    pub version: Option<String>,
    pub command: String,
    pub pack: String,
    pub shared_context: bool,
    pub skip_dev_deps: bool,
    pub environment: HashMap<String, String>,
    pub dirs: Vec<String>,
    pub include_deps: bool,
    pub image_name: String,
    pub base_image_arn: String,
    pub build_role_arn: String,
    pub bucket: String,
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

impl Build {
    pub fn new(
        dir: &str,
        runtime: &Runtime,
        bspec: Option<BuildSpec>,
        tasks: HashMap<String, String>,
        fname: &str,
    ) -> Build {
        match bspec {
            Some(b) => Build {
                dir: s!(dir),
                kind: b.kind,
                pre: b.pre,
                post: b.post,
                command: b.command,
                pack: b.pack.unwrap_or(String::from("echo 0")),
                version: b.version.clone(),
                shared_context: match b.shared_context {
                    Some(s) => s,
                    None => true,
                },
                skip_dev_deps: match b.skip_dev_deps {
                    Some(s) => s,
                    None => true,
                },
                environment: HashMap::new(),
                dirs: match b.dirs {
                    Some(d) => d,
                    None => vec![],
                },
                include_deps: match b.include_deps {
                    Some(d) => d,
                    None => false,
                },

                base_image_arn: match b.base_image_arn {
                    Some(d) => d,
                    None => format!("arn:aws:lambda:{{{{region}}}}:aws:microvm-image:al2023-1"),
                },

                build_role_arn: match b.build_role_arn {
                    Some(d) => d,
                    None => {
                        format!("arn:aws:iam::{{{{account}}}}:role/tc-base-microvm-{{{{sandbox}}}}")
                    }
                },

                image_name: match b.image_name {
                    Some(d) => d,
                    None => {
                        let version = match &b.version {
                            Some(v) => v.replace(".", "-"),
                            None => "0_1_0".to_string(),
                        };
                        format!("{}_{}_{{{{sandbox}}}}", fname, version)
                    }
                },

                // FIXME
                bucket: match b.bucket {
                    Some(d) => d,
                    None => match std::env::var("TC_ASSET_BUCKET") {
                        Ok(s) => s,
                        Err(_) => format!("{{{{ASSET_BUCKET}}}}"),
                    },
                },
            },
            None => {
                let command = match tasks.get("build") {
                    Some(c) => c.to_owned(),
                    None => s!("zip -9 -q lambda.zip *.*"),
                };

                Build {
                    dir: s!(dir),
                    kind: infer_kind(&runtime.package_type),
                    pre: vec![],
                    post: vec![],
                    version: None,
                    command: command,
                    pack: String::from("echo 0"),
                    shared_context: false,
                    skip_dev_deps: false,
                    environment: HashMap::new(),
                    dirs: vec![],
                    include_deps: false,
                    base_image_arn: String::from(""),
                    build_role_arn: String::from(""),
                    bucket: String::from(""),
                    image_name: String::from(""),
                }
            }
        }
    }
}
