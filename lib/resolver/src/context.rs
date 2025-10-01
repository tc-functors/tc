use configurator::Config;
use kit as u;
use provider::Auth;
use std::collections::HashMap;

fn abbr(name: &str) -> String {
    if name.chars().count() > 15 {
        u::abbreviate(name, "-")
    } else {
        name.to_string()
    }
}

pub struct Context {
    pub auth: Auth,
    pub namespace: String,
    pub sandbox: String,
    pub trace: bool,
    pub config: Config,
}

impl Context {
    pub fn render(&self, s: &str) -> String {
        let mut table: HashMap<&str, &str> = HashMap::new();
        let account = &self.auth.account;
        let region = &self.auth.region;
        let abbr_namespace = abbr(&self.namespace);

        let repo = match std::env::var("TC_ECR_REPO") {
            Ok(r) => &r.to_owned(),
            Err(_) => &self.config.aws.ecr.repo,
        };

        let lazy_id = format!("{{{{lazy_id}}}}");

        table.insert("account", account);
        table.insert("acc", account);
        table.insert("region", region);
        table.insert("namespace", &self.namespace);
        table.insert("abbr_namespace", &abbr_namespace);
        table.insert("sandbox", &self.sandbox);
        table.insert("env", &self.auth.name);
        table.insert("profile", &self.auth.name);
        table.insert("repo", repo);
        table.insert("lazy_id", &lazy_id);
        table.insert("API_GATEWAY_URL", "{{API_GATEWAY_URL}}");
        u::stencil(s, table)
    }
}
