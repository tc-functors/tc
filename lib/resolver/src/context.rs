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
    pub version: String
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

        let bucket = match std::env::var("TC_ASSET_BUCKET") {
            Ok(r) => &r.to_owned(),
            Err(_) => &self.config.aws.lambda.asset_bucket,
        };

        let asset_acc = match std::env::var("TC_ASSET_ACCOUNT") {
            Ok(r) => &r.to_owned(),
            Err(_) => if let Some(c) = &self.config.aws.lambda.asset_account {                c
            } else {
                account
            }
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
        table.insert("version", &self.version);
        table.insert("repo", repo);
        table.insert("lazy_id", &lazy_id);
        table.insert("API_GATEWAY_URL", "{{API_GATEWAY_URL}}");
        table.insert("GRAPHQL_ENDPOINT", "{{GRAPHQL_ENDPOINT}}");
        table.insert("GRAPHQL_API_KEY", "{{GRAPHQL_API_KEY}}");
        table.insert("HTTP_DOMAIN", "{{HTTP_DOMAIN}}");
        table.insert("API_KEY", "{{API_KEY}}");
        table.insert("ASSET_BUCKET", &bucket);
        table.insert("ASSET_ACCOUNT", &asset_acc);
        u::stencil(s, table)
    }
}
