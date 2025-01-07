use aws_sdk_apigatewayv2::types::{
    AuthorizationType, ConnectionType, IntegrationType, ProtocolType,
};
use aws_sdk_apigatewayv2::{Client, Error};
use std::collections::HashMap;

use super::Env;
use kit::*;
use log::{debug, info};

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

#[derive(Clone, Debug)]
pub struct Api {
    pub client: Client,
    pub name: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
    pub uri: String,
    pub role: String,
    pub path: String,
    pub method: String,
    pub authorizer: String,
    pub request_template: String,
    pub response_template: String,
}

impl Api {
    pub async fn create(self) -> String {
        let api = self.clone();
        let r = self
            .clone()
            .client
            .create_api()
            .name(api.name)
            .protocol_type(ProtocolType::Http)
            .send()
            .await
            .unwrap();
        r.api_id.unwrap()
    }

    pub async fn _delete(self, api_id: &str) {
        self.client
            .delete_api()
            .api_id(api_id)
            .send()
            .await
            .unwrap();
    }

    pub async fn find(&self) -> Option<String> {
        let r = self
            .client
            .get_apis()
            .max_results(s!("1000"))
            .send()
            .await
            .unwrap();
        let items = r.items;
        match items {
            Some(apis) => {
                for api in apis.to_vec() {
                    match api.name {
                        Some(name) => {
                            if name == self.name {
                                return api.api_id;
                            }
                        }
                        None => (),
                    }
                }
                return None;
            }
            None => None,
        }
    }

    pub async fn find_or_create(&self) -> String {
        let api_id = self.find().await;
        match api_id {
            Some(id) => {
                debug!("Found API {} id: {}", &self.name, &id);
                id
            }
            _ => {
                let id = self.clone().create().await;
                debug!("Created API {} id: {}", &self.name, &id);
                id
            }
        }
    }

    pub async fn find_route(&self, api_id: &str, route_key: &str) -> Option<String> {
        let r = self
            .client
            .get_routes()
            .api_id(api_id.to_string())
            .max_results(s!("2000"))
            .send()
            .await
            .unwrap();
        let items = r.items;
        match items {
            Some(routes) => {
                for route in routes.to_vec() {
                    match route.route_key {
                        Some(key) => {
                            if &key == route_key {
                                return route.route_id;
                            }
                        }
                        None => (),
                    }
                }
                return None;
            }
            None => None,
        }
    }

    async fn create_route(
        &self,
        api_id: &str,
        route_key: &str,
        target: &str,
        authorizer: Option<String>,
    ) -> String {
        match authorizer {
            Some(auth) => {
                let res = self
                    .client
                    .create_route()
                    .api_id(s!(api_id))
                    .route_key(route_key)
                    .target(target)
                    .authorization_type(AuthorizationType::Custom)
                    .authorizer_id(auth)
                    .send()
                    .await
                    .unwrap();
                res.route_id.unwrap()
            }
            _ => {
                let res = self
                    .client
                    .create_route()
                    .api_id(s!(api_id))
                    .route_key(route_key)
                    .target(target)
                    .send()
                    .await
                    .unwrap();
                res.route_id.unwrap()
            }
        }
    }

    pub async fn find_or_create_route(
        &self,
        api_id: &str,
        integration_id: &str,
        authorizer_id: Option<String>,
    ) -> String {
        let route_key = strip(&format!("{} {}", self.method, self.path), "/");
        let target = format!("integrations/{}", integration_id);
        let maybe_route = self.find_route(api_id, &route_key).await;

        match maybe_route {
            Some(rid) => {
                debug!("Found route key {}", &route_key);
                rid
            }
            None => {
                let rid = self
                    .create_route(api_id, &route_key, &target, authorizer_id)
                    .await;
                info!("Created route key {}", &route_key);
                rid
            }
        }
    }

    async fn find_lambda_integration(&self, api_id: &str, lambda_arn: &str) -> Option<String> {
        let r = self
            .client
            .get_integrations()
            .api_id(api_id.to_string())
            .max_results(s!("2000"))
            .send()
            .await
            .unwrap();
        let items = r.items;
        match items {
            Some(ints) => {
                for int in ints.to_vec() {
                    match int.integration_uri {
                        Some(uri) => {
                            if uri == lambda_arn {
                                return int.integration_id;
                            }
                        }
                        None => (),
                    };
                }
                None
            }
            None => None,
        }
    }

    async fn create_lambda_integration(
        &self,
        api_id: &str,
        lambda_arn: &str,
    ) -> Result<String, Error> {
        let api = self.clone();
        let res = self
            .client
            .create_integration()
            .api_id(s!(api_id))
            .connection_type(ConnectionType::Internet)
            .credentials_arn(api.role)
            .payload_format_version(s!("2.0"))
            .integration_type(IntegrationType::AwsProxy)
            .integration_uri(lambda_arn)
            .send()
            .await;
        match res {
            Ok(r) => Ok(r.integration_id.unwrap()),
            Err(e) => panic!("{:?}", e),
        }
    }

    pub async fn find_or_create_lambda_integration(
        &self,
        api_id: &str,
        lambda_arn: &str,
    ) -> String {
        let maybe_int = self.find_lambda_integration(api_id, lambda_arn).await;
        match maybe_int {
            Some(id) => {
                //info!("Found Lambda Integration {}", id);
                id
            }
            _ => {
                let id = self
                    .create_lambda_integration(api_id, lambda_arn)
                    .await
                    .unwrap();
                //info!("Created Lambda Integration {}", id);
                id
            }
        }
    }

    async fn find_sfn_integration(&self, api_id: &str) -> Option<String> {
        let r = self
            .client
            .get_integrations()
            .api_id(api_id.to_string())
            .max_results(s!("1000"))
            .send()
            .await
            .unwrap();
        let items = r.items;
        match items {
            Some(ints) => {
                for int in ints.to_vec() {
                    match int.request_parameters {
                        Some(req) => match req.get("Name") {
                            Some(name) => {
                                if name == &format!("sfn-{}", self.method) {
                                    return int.integration_id;
                                }
                            }
                            None => (),
                        },
                        None => (),
                    }
                }
                return None;
            }
            None => None,
        }
    }

    async fn create_sfn_integration(&self, api_id: &str, sfn_arn: &str) -> Result<String, Error> {
        let api = self.clone();
        let mut req: HashMap<String, String> = HashMap::new();
        req.insert("StateMachineArn".to_string(), s!(sfn_arn));
        req.insert("Name".to_string(), format!("sfn-{}", self.method));
        if &self.method == "POST" {
            req.insert("Input".to_string(), "{\"path\": \"${request.path}\", \"detail\": ${request.body.detail}, \"method\": \"${context.httpMethod}\"}".to_string());
        } else {
            req.insert(
                "Input".to_string(),
                "{\"path\": \"${request.path}\", \"method\": \"${context.httpMethod}\"}"
                    .to_string(),
            );
        }

        let res = self
            .client
            .create_integration()
            .api_id(s!(api_id))
            .connection_type(ConnectionType::Internet)
            .credentials_arn(api.role)
            .payload_format_version(s!("2.0"))
            .integration_type(IntegrationType::AwsProxy)
            .integration_subtype(s!("StepFunctions-StartSyncExecution"))
            .set_request_parameters(Some(req))
            .send()
            .await;
        match res {
            Ok(r) => Ok(r.integration_id.unwrap()),
            Err(e) => panic!("{:?}", e),
        }
    }

    pub async fn find_or_create_sfn_integration(&self, api_id: &str, sfn_arn: &str) -> String {
        let maybe_int = self.find_sfn_integration(api_id).await;
        match maybe_int {
            Some(id) => id,
            _ => self.create_sfn_integration(api_id, sfn_arn).await.unwrap(),
        }
    }

    pub async fn create_deployment(&self, api_id: &str, stage: &str) {
        self.client
            .create_deployment()
            .api_id(api_id)
            .stage_name(stage)
            .send()
            .await
            .unwrap();
    }

    pub async fn delete_route(&self, api_id: &str, route_id: &str) -> Result<(), Error> {
        let res = self
            .client
            .delete_route()
            .api_id(api_id)
            .route_id(route_id)
            .send()
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn find_authorizer(&self, api_id: &str) -> Option<String> {
        let r = self
            .client
            .get_authorizers()
            .api_id(s!(api_id))
            .send()
            .await
            .unwrap();
        let items = r.items;
        match items {
            Some(auths) => {
                for auth in auths.to_vec() {
                    match auth.name {
                        Some(name) => {
                            if name == self.authorizer {
                                return auth.authorizer_id;
                            }
                        }
                        None => (),
                    }
                }
                return None;
            }
            None => None,
        }
    }

    pub async fn create_stage(&self, api_id: &str) -> Result<String, Error> {
        let stage = self.clone().stage;
        let stage_variables = self.stage_variables.to_owned();
        info!("Creating stage {}", &stage);
        let res = self
            .client
            .create_stage()
            .api_id(s!(api_id))
            .stage_name(stage.clone())
            .set_stage_variables(Some(stage_variables))
            .send()
            .await;

        match res {
            Ok(_) => Ok(stage),
            _ => Ok(stage),
        }
    }
}
