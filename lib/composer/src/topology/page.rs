use crate::spec::{
    ConfigSpec,
    PageSpec,
    TopologySpec,
};
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyStatement {
    #[serde(rename(serialize = "Sid"), alias = "Sid")]
    pub sid: String,
    #[serde(rename(serialize = "Effect"), alias = "Effect")]
    pub effect: String,
    #[serde(rename(serialize = "Principal"), alias = "Principal")]
    pub principal: HashMap<String, String>,
    #[serde(rename(serialize = "Action"), alias = "Action")]
    pub action: String,
    #[serde(rename(serialize = "Resource"), alias = "Resource")]
    pub resource: String,
    #[serde(rename(serialize = "Condition"), alias = "Condition")]
    pub condition: HashMap<String, HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BucketPolicy {
    #[serde(rename(serialize = "Version"), alias = "Version")]
    pub version: String,
    #[serde(rename(serialize = "Id"), alias = "Id")]
    pub id: String,
    #[serde(rename(serialize = "Statement"), alias = "Statement")]
    pub statement: Vec<PolicyStatement>,
}

impl BucketPolicy {
    fn new(namespace: &str, name: &str, bucket: &str) -> BucketPolicy {
        let mut principal: HashMap<String, String> = HashMap::new();
        principal.insert(s!("Service"), s!("cloudfront.amazonaws.com"));

        let mut condition: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut cond_exp: HashMap<String, String> = HashMap::new();
        cond_exp.insert(
            s!("AWS:SourceArn"),
            format!("arn:aws:cloudfront::{{{{account}}}}:distribution/{{{{lazy_id}}}}"),
        );
        condition.insert(s!("StringEquals"), cond_exp);

        let statement = PolicyStatement {
            sid: format!("AllowCloudFront{}{}", namespace, name),
            effect: s!("Allow"),
            principal: principal,
            action: s!("s3:GetObject"),
            resource: format!("arn:aws:s3:::{}/{}/{}/*", bucket, namespace, name),
            condition: condition,
        };

        BucketPolicy {
            version: s!("2008-10-17"),
            id: s!("OSSPolicyForCloudFrontPrivateContent"),
            statement: vec![statement],
        }
    }

    pub fn add_statement(&self, existing_policy: &str) -> BucketPolicy {
        let mut ex: BucketPolicy = serde_json::from_str(&existing_policy).unwrap();
        let mut xs: Vec<PolicyStatement> = ex.statement.clone();
        let current_id = &self.statement.first().unwrap().sid;
        for s in ex.statement {
            if s.sid != *current_id {
                xs.extend(self.statement.clone());
            }
        }
        ex.statement = xs;
        ex
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Infra {
    pub bucket: Option<String>,
    pub domains: Option<Vec<String>>,
}

impl Infra {
    pub fn new(infra_dir: &str, _namespace: &str, name: &str) -> Option<Infra> {
        let f = format!("{}/pages/{}.json", infra_dir, name);
        if u::file_exists(&f) {
            let data: String = u::slurp(&f);
            let inf: Infra = serde_json::from_str(&data).unwrap();
            Some(inf)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Page {
    pub namespace: String,
    pub dir: String,
    pub dist: String,
    pub build: Option<String>,
    pub caller_ref: String,
    pub bucket: String,
    pub bucket_policy: BucketPolicy,
    pub bucket_prefix: String,
    pub origin_paths: HashMap<String, String>,
    pub origin_domain: String,
    pub default_root_object: String,
    pub domains: Vec<String>,
}

fn find_bucket(
    given_bucket: &Option<String>,
    config: &ConfigSpec,
    infra: &Option<Infra>,
) -> String {
    match given_bucket {
        Some(b) => b.to_string(),
        None => match infra {
            Some(inf) => {
                if let Some(bucket) = &inf.bucket {
                    bucket.to_string()
                } else {
                    match &config.aws.cloudfront.bucket {
                        Some(c) => c.to_string(),
                        None => match std::env::var("TC_PAGES_BUCKET") {
                            Ok(e) => e,
                            Err(_) => panic!("No bucket configured"),
                        },
                    }
                }
            }
            None => match &config.aws.cloudfront.bucket {
                Some(c) => c.to_string(),
                None => match std::env::var("TC_PAGES_BUCKET") {
                    Ok(e) => e,
                    Err(_) => panic!("No bucket configured"),
                },
            },
        },
    }
}

fn find_domains(given_domains: &Option<Vec<String>>, infra: &Option<Infra>) -> Vec<String> {
    match given_domains {
        Some(d) => d.to_vec(),
        None => {
            if let Some(inf) = infra {
                if let Some(domains) = &inf.domains {
                    domains.to_vec()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        }
    }
}

fn make_paths(namespace: &str, name: &str) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let p = format!("/{}/{}", namespace, name);
    let id = format!("{}", name);
    h.insert(id, p);
    h
}

fn make(
    name: &str,
    namespace: &str,
    ps: &PageSpec,
    infra: &Option<Infra>,
    config: &ConfigSpec,
) -> Page {
    let bucket = find_bucket(&ps.bucket, config, infra);
    let origin_domain = format!("{}.s3.{{{{region}}}}.amazonaws.com", &bucket);
    let bucket_policy = BucketPolicy::new(namespace, name, &bucket);
    let caller_ref = format!("{}-{}", namespace, name);
    let dir = u::maybe_string(ps.dir.clone(), &u::pwd());
    let build = match &ps.build {
        Some(bs) => Some(bs.join(" && ")),
        None => None,
    };
    let dist = u::maybe_string(ps.dist.clone(), "dist");

    let paths = make_paths(namespace, name);

    Page {
        namespace: namespace.to_string(),
        dir: dir,
        dist: dist,
        build: build,
        caller_ref: caller_ref,
        bucket_policy: bucket_policy,
        bucket_prefix: format!("{}/{}", namespace, name),
        bucket: bucket,
        origin_domain: origin_domain,
        origin_paths: paths,
        default_root_object: s!("index.html"),
        domains: find_domains(&ps.domains, infra),
    }
}

pub fn make_all(
    spec: &TopologySpec,
    infra_dir: &str,
    config: &ConfigSpec,
) -> HashMap<String, Page> {
    let mut h: HashMap<String, Page> = HashMap::new();
    if let Some(pspec) = &spec.pages {
        for (name, ps) in pspec {
            let maybe_infra = Infra::new(infra_dir, &spec.name, &name);
            let page = make(&name, &spec.name, ps, &maybe_infra, config);
            h.insert(name.to_string(), page);
        }
    }
    h
}
