use kit as u;
use kit::*;
use std::{
    collections::HashMap,
    env,
    panic,
};

#[derive(Clone, Debug)]
pub struct Circle {
    pub repo: String,
    pub org: String,
    pub token: String,
}

fn find_org() -> String {
    match std::env::var("GITHUB_ORG") {
        Ok(k) => k,
        Err(_) => {
            let s1 = u::sh("git config --get remote.origin.url", &u::pwd());
            let s2 = u::second(&s1, ":");
            u::split_first(&s2, "/")
        }
    }
}

impl Circle {
    pub fn init(repo: &str) -> Circle {
        let token = match env::var("CIRCLE_CI_TOKEN") {
            Ok(v) => v,
            Err(_e) => {
                panic::set_hook(Box::new(|_| {
                    println!("Please set CIRCLE_CI_TOKEN env variable");
                }));
                panic!("CIRCLE_CI_TOKEN envvar not found")
            }
        };



        Circle {
            repo: String::from(repo),
            org: find_org(),
            token: token,
        }
    }

    fn url(&self) -> String {
        //FIXME: parameterize repo
        format!(
            "https://circleci.com/api/v2/project/github/{}/{}/pipeline",
            self.org,
            self.repo
        )
    }

    fn headers(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert(s!("circle-token"), self.token.clone());
        h.insert(s!("content-type"), s!("application/json"));
        h.insert(s!("accept"), s!("application/json"));
        h.insert(
            s!("user-agent"),
            s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
        );
        h
    }

    pub async fn trigger_workflow(&self, payload: String) -> String {
        let url = &self.url();
        let res = u::http_post(url, self.headers(), payload).await.unwrap();
        println!("{:?}", &res);
        let num = res["number"].to_string();
        self.workflow_url(&num)
    }

    pub fn workflow_url(&self, num: &str) -> String {
        format!(
            "https://app.circleci.com/pipelines/github/{}/{}/{}",
            self.org,
            self.repo,
            num
        )
    }
}

pub async fn trigger_release(repo: &str, prefix: &str, version: &str, suffix: &str) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-release-version-suffix": "{suffix}",
              "tc-release-service": "{prefix}",
              "api_call": true
           }}}}"#
    );
    println!("Triggering release {}-{}", prefix, version);
    ci.trigger_workflow(payload).await
}

pub async fn trigger_tag(
    repo: &str,
    env: &str,
    sandbox: &str,
    prefix: &str,
    version: &str,
) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-deploy-service": "{prefix}",
              "tc-deploy-version": "{version}",
              "tc-deploy-sandbox": "{sandbox}",
              "tc-deploy-env": "{env}",
              "api_call": true
           }}}}"#
    );
    println!(
        "Triggering tag deploy {}:{}:{}/{}",
        env, sandbox, prefix, version
    );
    ci.trigger_workflow(payload).await
}

pub async fn trigger_branch(
    repo: &str,
    env: &str,
    sandbox: &str,
    prefix: &str,
    branch: &str,
) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-deploy-service": "{prefix}",
              "tc-deploy-branch": "{branch}",
              "tc-deploy-only-branch": true,
              "tc-deploy-sandbox": "{sandbox}",
              "tc-deploy-env": "{env}",
              "api_call": true
           }}}}"#
    );
    println!(
        "Triggering branch deploy {}:{}:{}/{}",
        env, sandbox, prefix, branch
    );
    ci.trigger_workflow(payload).await
}

pub async fn trigger_create(
    repo: &str,
    env: &str,
    sandbox: &str,
    dir: &str,
    branch: &str
) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-deploy-dir": "{dir}",
              "tc-create-from-dir": true,
              "tc-deploy-sandbox": "{sandbox}",
              "tc-build-branch": "{branch}",
              "tc-deploy-env": "{env}",
              "api_call": true
           }}}}"#
    );
    println!(
        "Triggering create from dir deploy {}:{} {}",
        env, sandbox, dir
    );
    ci.trigger_workflow(payload).await
}

pub async fn trigger_update(
    repo: &str,
    env: &str,
    sandbox: &str,
    dir: &str,
    branch: &str
) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-deploy-dir": "{dir}",
              "tc-update-from-dir": true,
              "tc-deploy-sandbox": "{sandbox}",
              "tc-build-branch": "{branch}",
              "tc-deploy-env": "{env}",
              "api_call": true
           }}}}"#
    );
    println!(
        "Triggering update from dir deploy {}:{} {}",
        env, sandbox, dir
    );
    ci.trigger_workflow(payload).await
}


pub async fn trigger_build(repo: &str, prefix: &str, function: &str, branch: &str) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-build-from-dir": true,
              "tc-build-branch": "{branch}",
              "tc-build-function": "{function}",
              "api_call": true
           }}}}"#
    );
    println!("Triggering build {} {}:{}", prefix, function, branch);
    ci.trigger_workflow(payload).await
}


pub async fn trigger_pipeline(repo: &str, env: &str, sandbox: &str) -> String {
    let ci = Circle::init(repo);
    let payload = format!(
        r#"
           {{
             "branch": "main",
             "parameters": {{
              "tc-deploy-snapshot-pipeline": true,
              "tc-deploy-sandbox": "{sandbox}",
              "tc-deploy-env": "{env}",
              "api_call": true
           }}}}"#
    );
    println!("Triggering snapshot pipeline");
    ci.trigger_workflow(payload).await
}
