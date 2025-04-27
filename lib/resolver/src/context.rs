use kit as u;
use provider::Env;
use std::collections::HashMap;

fn abbr(name: &str) -> String {
    if name.chars().count() > 15 {
        u::abbreviate(name, "-")
    } else {
        name.to_string()
    }
}

pub struct Context {
    pub env: Env,
    pub namespace: String,
    pub sandbox: String,
    pub trace: bool,
}

impl Context {
    pub fn render(&self, s: &str) -> String {
        let mut table: HashMap<&str, &str> = HashMap::new();
        let account = &self.env.account();
        let region = &self.env.region();
        let abbr_namespace = abbr(&self.namespace);

        let repo = match std::env::var("TC_ECR_REPO") {
            Ok(r) => &r.to_owned(),
            Err(_) => &self.env.config.aws.ecr.repo,
        };

        table.insert("account", account);
        table.insert("acc", account);
        table.insert("region", region);
        table.insert("namespace", &self.namespace);
        table.insert("abbr_namespace", &abbr_namespace);
        table.insert("sandbox", &self.sandbox);
        table.insert("env", &self.env.name);
        table.insert("profile", &self.env.name);
        table.insert("repo", repo);
        u::stencil(s, table)
    }
}
