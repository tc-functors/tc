use serde_derive::{Deserialize, Serialize};

static POLICY_VERSION: &str = "2012-10-17"; // override if necessary

#[derive(Serialize, Deserialize, Debug)]
pub enum Method {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "*PUT")]
    Put,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "OPTIONS")]
    Options,
    #[serde(rename = "*")]
    All,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct IAMPolicyStatement {
    pub Action: Vec<String>,
    pub Effect: Effect,
    pub Resource: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct AuthorizerPolicy {
    pub Version: String,
    pub Statement: Vec<IAMPolicyStatement>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Policy {
    pub region: String,
    pub aws_account_id: String,
    pub rest_api_id: String,
    pub stage: String,
    pub policy: AuthorizerPolicy,
}


impl Policy {
    pub fn new(region: &str, account_id: &str, api_id: &str, stage: &str) -> Policy {
        Policy {
            region: region.to_string(),
            aws_account_id: account_id.to_string(),
            rest_api_id: api_id.to_string(),
            stage: stage.to_string(),
            policy: AuthorizerPolicy {
                Version: POLICY_VERSION.to_string(),
                Statement: vec![],
            },
        }
    }

    pub fn add_method<T: Into<String>>(
        mut self,
        effect: Effect,
        method: Method,
        resource: T,
    ) -> Policy {
        let resource_arn = format!(
            "arn:aws:execute-api:{}:{}:{}/*/*/*",
            &self.region,
            &self.aws_account_id,
            &self.rest_api_id,
            // &self.stage,
            // serde_json::to_string(&method).unwrap(),
            // resource.into().trim_start_matches("/")
        );

        let stmt = IAMPolicyStatement {
            Effect: effect,
            Action: vec!["execute-api:Invoke".to_string()],
            Resource: vec![resource_arn],
        };

        self.policy.Statement.push(stmt);
        self
    }

    pub fn allow_all_methods(self) -> Self {
        self.add_method(Effect::Allow, Method::All, "*")
    }

    pub fn deny_all_methods(self) -> Self {
        self.add_method(Effect::Deny, Method::All, "*")
    }

    pub fn allow_method(self, method: Method, resource: String) -> Self {
        self.add_method(Effect::Allow, method, resource)
    }

    pub fn deny_method(self, method: Method, resource: String) -> Self {
        self.add_method(Effect::Deny, method, resource)
    }

    // Creates and executes a new child thread.
    pub fn build(self) -> AuthorizerPolicy {
        self.policy
    }
}
