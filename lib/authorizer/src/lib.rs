mod aws;
mod gcp;

use std::env::var;
use aws_config::SdkConfig;

#[derive(Clone, Debug)]
pub struct Auth {
    pub name: String,
    pub assume_role: Option<String>,
    pub aws_config: SdkConfig,
    pub account: String,
    pub region: String
}

impl Auth {

    pub async fn new(
        profile: Option<String>,
        assume_role: Option<String>
    ) -> Auth {

        let name = match profile {
            Some(p) => p,
            None => "default".to_string()
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
            region: region
        }
    }

    pub async fn inherit(&self, profile: Option<String>) -> Auth {
        match profile {
            Some(_) => {
                let role = match var("TC_CENTRALIZED_ASSUME_ROLE") {
                    Ok(r) => Some(r),
                    Err(_) => self.assume_role.clone(),
                };
                Auth::new(profile, role).await
            }
            None => match std::env::var("AWS_PROFILE") {
                Ok(p) => {
                    let role = match var("TC_CENTRALIZED_ASSUME_ROLE") {
                        Ok(r) => Some(r),
                        Err(_) => self.assume_role.clone(),
                    };
                    Auth::new(Some(p), role).await
                }
                Err(_) => self.clone(),
            },
        }
    }


    pub fn sfn_uri(&self) -> String {
        format!(
            "arn:aws:apigateway:{}:states:action/StartSyncExecution",
            &self.region
        )
    }

    pub fn sfn_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:states:{}:{}:stateMachine:{}",
            &self.region,
            self.account,
            name
        )
    }

    pub fn lambda_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:lambda:{}:{}:function:{}",
            &self.region,
            &self.account,
            name
        )
    }

    pub fn layer_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:lambda:{}:{}:layer:{}",
            &self.region,
            &self.account,
            name
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
            &self.region,
            &self.account,
            bus_name
        )
    }

    pub fn api_endpoint(&self, api_id: &str, stage: &str) -> String {
        format!(
            "https://{}.execute-api.{}.amazonaws.com/{}",
            api_id,
            self.region,
            stage
        )
    }

    pub fn sfn_url(&self, name: &str, id: &str) -> String {
        format!(
            "https://{}.console.aws.amazon.com/states/home?region={}#/v2/executions/details/arn:aws:states:{}:{}:execution:{}:{}",
            &self.region,
            &self.region,
            &self.region,
            &self.account,
            name,
            id
        )
    }

    pub fn sfn_url_express(&self, arn: &str) -> String {
        format!(
            "https://{}.console.aws.amazon.com/states/home?region={}#/express-executions/details/{}?startDate={}",
            &self.region,
            &self.region,
            arn,
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
            &self.region,
            &self.account,
            api_id
        )
    }

    pub fn graphql_arn(&self, id: &str) -> String {
        format!(
            "arn:aws:appsync:{}:{}:endpoints/graphql-api/{}",
            &self.region,
            &self.account,
            id
        )
    }

    pub fn sqs_arn(&self, name: &str) -> String {
        format!(
            "arn:aws:sqs:{}:{}:{}",
            &self.region,
            &self.account,
            name
        )
    }

    pub fn sqs_url(&self, name: &str) -> String {
        format!(
            "https://sqs.{}.amazonaws.com/{}/{}",
            &self.region,
            &self.account,
            name
        )
    }

}
