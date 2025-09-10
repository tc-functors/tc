mod aws;
mod gcp;

use aws_config::SdkConfig;
use aws_sdk_sts::config::ProvideCredentials;

#[derive(Clone, Debug)]
pub struct Auth {
    pub name: String,
    pub assume_role: Option<String>,
    pub aws_config: SdkConfig,
    pub account: String,
    pub region: String,
}

impl Auth {
    pub async fn new(profile: Option<String>, assume_role: Option<String>) -> Auth {
        let name = match profile {
            Some(p) => p,
            None => "default".to_string(),
        };

        let config = aws::get_config(&name, assume_role.clone()).await;

        let client = aws::make_client(&config).await;
        let account = aws::get_account_id(&client).await;
        let region = aws::get_region();

        Auth {
            name: name,
            assume_role: assume_role,
            aws_config: config,
            account: account,
            region: region,
        }
    }

    pub async fn assume(&self, profile: Option<String>, assume_role: Option<String>) -> Auth {
        match profile {
            Some(_) => match std::env::var("TC_ASSUME_ROLE") {
                Ok(_) => Auth::new(profile, assume_role).await,
                Err(_) => Auth::new(profile, None).await,
            },
            None => self.clone(),
        }
    }

    pub async fn get_keys(&self) -> (String, String, String) {
        let config = &self.aws_config;
        let provider = config.credentials_provider().unwrap();
        let credentials = provider.provide_credentials().await.unwrap();
        let key = credentials.access_key_id();
        let secret = credentials.secret_access_key();
        let session_token = credentials.session_token().unwrap_or_default();

        (
            key.to_string(),
            secret.to_string(),
            session_token.to_string(),
        )
    }

    pub fn sfn_uri(&self) -> String {
        format!(
            "arn:aws:apigateway:{}:states:action/StartSyncExecution",
            &self.region
        )
    }

    pub fn lambda_uri(&self, name: &str) -> String {
        format!(
            "arn:aws:apigateway:{}:lambda:path/2015-03-31/functions/arn:aws:lambda:{}:{}:function:{}/invocations",
            &self.region, &self.region, &self.account, name
        )
    }

    pub fn sfn_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:states:{}:{}:stateMachine:{}",
            &self.region, self.account, name
        )
    }

    pub fn sfn_exec_arn(&self, name: &str, id: &str) -> String {
        format!(
            "arn:aws:states:{}:{}:execution:{}:{}",
            &self.region, self.account, name, id
        )
    }

    pub fn lambda_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:lambda:{}:{}:function:{}",
            &self.region, &self.account, name
        )
    }

    pub fn layer_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:lambda:{}:{}:layer:{}",
            &self.region, &self.account, name
        )
    }

    pub fn role_arn(&self, name: &str) -> String {
        format!("arn:aws:iam::{}:role/{}", &self.account, name)
    }

    pub fn policy_arn(&self, name: &str) -> String {
        format!("arn:aws:iam::{}:policy/{}", &self.account, name)
    }

    pub fn event_bus_arn(&self, bus_name: &str) -> String {
        format!(
            "arn:aws:events:{}:{}:event-bus/{}",
            &self.region, &self.account, bus_name
        )
    }

    pub fn api_endpoint(&self, api_id: &str, _stage: &str) -> String {
        format!(
            "https://{}.execute-api.{}.amazonaws.com",
            api_id, self.region
        )
    }

    pub fn sfn_url(&self, name: &str, id: &str) -> String {
        format!(
            "https://{}.console.aws.amazon.com/states/home?region={}#/v2/executions/details/arn:aws:states:{}:{}:execution:{}:{}",
            &self.region, &self.region, &self.region, &self.account, name, id
        )
    }

    pub fn sfn_url_express(&self, arn: &str) -> String {
        let express_exec_arn = arn.replace("stateMachine", "express");

        format!(
            "https://{}.console.aws.amazon.com/states/home?region={}#/express-executions/details/{}?startDate={}",
            &self.region,
            &self.region,
            express_exec_arn,
            kit::current_millis() - 200000
        )
    }

    //config
    pub fn base_role(&self, name: &str) -> String {
        format!("tc-base-{}-role", name)
    }

    pub fn base_policy(&self, name: &str) -> String {
        format!("tc-base-{}-policy", name)
    }

    pub fn api_integration_arn(&self, lambda_arn: &str) -> String {
        format!(
            "arn:aws:apigateway:us-west-2:lambda:path/2015-03-31/functions/{}/invocations",
            lambda_arn
        )
    }

    pub fn api_arn(&self, api_id: &str) -> String {
        format!(
            "arn:aws:execute-api:{}:{}:{}/*/*/*",
            &self.region, &self.account, api_id
        )
    }

    pub fn pool_arn(&self, pool_id: &str) -> String {
        format!(
            "arn:aws:cognito-idp:{}:{}:userpool/{}",
            &self.region, &self.account, pool_id
        )
    }

    pub fn authorizer_arn(&self, api_id: &str, _name: &str) -> String {
        format!(
            "arn:aws:execute-api:{}:{}:{}/*/*",
            &self.region, &self.account, api_id,
        )
    }

    pub fn graphql_arn(&self, id: &str) -> String {
        format!(
            "arn:aws:appsync:{}:{}:endpoints/graphql-api/{}",
            &self.region, &self.account, id
        )
    }

     pub fn graphql_api_arn(&self, id: &str) -> String {
        format!(
            "arn:aws:appsync:{}:{}:apis/{}",
            &self.region, &self.account, id
        )
    }

    pub fn sqs_arn(&self, name: &str) -> String {
        format!("arn:aws:sqs:{}:{}:{}", &self.region, &self.account, name)
    }

    pub fn sqs_url(&self, name: &str) -> String {
        format!(
            "https://sqs.{}.amazonaws.com/{}/{}",
            &self.region, &self.account, name
        )
    }
}
