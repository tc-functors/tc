use std::collections::HashMap;

use crate::spec::TopologySpec;
use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BucketPolicy {


}

impl BucketPolicy {
    fn new() -> BucketPolicy {

    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Page {
    pub dir: String,
    pub dist: String,
    pub build: String,
    pub caller_ref: String,
    pub bucket: String,
    pub bucket_policy: String,
    pub origin_paths: Vec<String>,
    pub origin_domain: String,
    pub default_root_object: String,
}


fn find_bucket(given_bucket: Option<String>, config: &ConfigSpec) -> String {
    match given_bucket {
        Some(b) => b,
        None => match config.cloudfront.bucket {
            Some(c) => c,
            None => panic!("No bucket configured")
        }
    }
}

fn make(ps: &PageSpec, config: &ConfigSpec) -> Page {
    let bucket = find_bucket(ps.bucket, config);
    let origin_domain = format!("{}.s3.amazonaws.com", &bucket);
    let bucket_policy = BucketPolicy::new();

    let page = Page {
        dist: s.dist.clone(),
        build: s.build.clone(),
        bucket: bucket,
        bucket_policy: bucket_policy,
    };
}

pub fn make_all(spec: &TopologySpec, config: &ConfigSpec) -> HashMap<String, Page> {
    let mut h: HashMap<String, Page> = HashMap::new();
    if let Some(pspec) = &spec.pages {
        for (name, ps) in pspec {
            let page = make(ps, config);
            h.insert(name.to_string(), page);
        }
    }
    h
}
