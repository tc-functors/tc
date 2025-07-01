use std::collections::HashMap;

use crate::spec::{ConfigSpec, TopologySpec, PageSpec};

use serde_derive::{
    Deserialize,
    Serialize,
};

use kit::*;
use kit as u;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyStatement {
    #[serde(rename(serialize = "Sid"))]
    sid: String,
    #[serde(rename(serialize = "Effect"))]
    effect: String,
    #[serde(rename(serialize = "Principal"))]
    principal: HashMap<String, String>,
    #[serde(rename(serialize = "Action"))]
    action: String,
    #[serde(rename(serialize = "Resource"))]
    resource: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BucketPolicy {
    #[serde(rename(serialize = "Version"))]
    version: String,
    #[serde(rename(serialize = "Id"))]
    id: String,
    #[serde(rename(serialize = "Statement"))]
    statement: PolicyStatement,
}

impl BucketPolicy {
    fn new(bucket: &str) -> BucketPolicy {

        let mut principal: HashMap<String, String> = HashMap::new();
        principal.insert(s!("AWS"),
                         format!("arn:aws:iam::cloudfront:user/CloudFront Origin Access Identity {{{{oai_id}}}}"));
        let statement = PolicyStatement {
            sid: s!("AllowCloudFrontServicePrincipalWithOAI"),
            effect: s!("Allow"),
            principal: principal,
            action: s!("s3:GetObject"),
            resource: format!("arn:aws:s3:::{}/*", bucket)
        };

        BucketPolicy {
            version: s!("2008-10-17"),
            id: s!("OSSPolicyForCloudFrontPrivateContent"),
            statement: statement
        }

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
    pub bucket_prefix: String,
    pub origin_paths: Vec<String>,
    pub origin_domain: String,
    pub default_root_object: String,
    pub domains: Vec<String>
}


fn find_bucket(given_bucket: &Option<String>, config: &ConfigSpec) -> String {
    match given_bucket {
        Some(b) => b.to_string(),
        None => match &config.aws.cloudfront.bucket {
            Some(c) => c.to_string(),
            None => panic!("No bucket configured")
        }
    }
}

fn find_domains(given_domains: &Option<Vec<String>>, infra_dir: &str) -> Vec<String> {
    match given_domains {
        Some(d) => d.to_vec(),
        None => {
            // FIXME: get it from infra/pages/<page-name>.json
            vec![]
        }
    }
}

fn make(name: &str, ps: &PageSpec, infra_dir: &str, config: &ConfigSpec) -> Page {
    let bucket = find_bucket(&ps.bucket, config);
    let origin_domain = format!("{}.{{{{region}}}}.s3.amazonaws.com", &bucket);
    let bucket_policy = BucketPolicy::new(&bucket);
    let caller_ref = format!("{}-{}", &bucket, name);
    let dir = u::maybe_string(ps.dir.clone(), &u::pwd());
    let build = u::maybe_string(ps.build.clone(), "npm build");
    let dist = u::maybe_string(ps.dist.clone(), "dist");

    let paths = match &ps.paths {
        Some(p) => p.clone(),
        None => vec![]
    };

    Page {
        dir: dir,
        dist: dist,
        build: build,
        caller_ref: caller_ref,
        bucket_policy: serde_json::to_string(&bucket_policy).unwrap(),
        bucket_prefix: format!("{}/{{{{sandbox}}}}", &bucket),
        bucket: bucket,
        origin_domain: origin_domain,
        origin_paths: paths,
        default_root_object: s!("index.html"),
        domains: find_domains(&ps.domains, infra_dir)
    }
}

pub fn make_all(spec: &TopologySpec, infra_dir: &str, config: &ConfigSpec) -> HashMap<String, Page> {
    let mut h: HashMap<String, Page> = HashMap::new();
    if let Some(pspec) = &spec.pages {
        for (name, ps) in pspec {
            let page = make(&name, ps, infra_dir, config);
            h.insert(name.to_string(), page);
        }
    }
    h
}
