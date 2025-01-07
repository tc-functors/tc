use crate::compiler::Config;
use aws::Env;
use kit as u;
use std::collections::HashMap;

pub struct Context {
    pub env: Env,
    pub namespace: String,
    pub sandbox: String,
    pub name: String,
    pub resolve: bool,
    pub config: Config,
}

impl Context {
    pub fn render(&self, s: &str) -> String {
        let mut table: HashMap<&str, &str> = HashMap::new();
        let account = &self.env.account();
        let region = &self.env.region();
        table.insert("account", account);
        table.insert("region", region);
        table.insert("namespace", &self.namespace);
        table.insert("sandbox", &self.sandbox);
        table.insert("env", &self.env.name);
        u::stencil(s, table)
    }

    pub fn resolve_config(&mut self) {
        let cs = serde_json::to_string(&self.config).unwrap();
        let csr = self.render(&cs);
        let cfg: Config = serde_json::from_str(&csr).expect("fail");
        self.config = cfg;
    }
}
