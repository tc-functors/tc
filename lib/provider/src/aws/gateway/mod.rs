use crate::Env;
use aws_sdk_apigatewayv2::{
    Client,
    Error,
    types::{
        AuthorizationType,
        ProtocolType,
    },
};
use kit::*;
use std::collections::HashMap;

mod lambda;
mod sfn;
// mod eventbridge;
// mod sqs;

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

    pub async fn delete(self, api_id: &str) {
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
                println!("Found API {} id: {}", &self.name, &id);
                id
            }
            _ => {
                let id = self.clone().create().await;
                println!("Created API {} id: {}", &self.name, &id);
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
    ) {
        let route_key = strip(&format!("{} {}", self.method, self.path), "/");
        let target = format!("integrations/{}", integration_id);
        let maybe_route = self.find_route(api_id, &route_key).await;

        match maybe_route {
            Some(_) => {
                tracing::debug!("Found route key {}", &route_key);
            }
            None => {
                self
                    .create_route(api_id, &route_key, &target, authorizer_id)
                    .await;
                tracing::debug!("Created route key {}", &route_key);
            }
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

    pub async fn create_authorizer(&self) {
        todo!()
    }

    pub async fn create_stage(&self, api_id: &str) {
        let stage = self.clone().stage;
        let stage_variables = self.stage_variables.to_owned();
        println!("Creating gateway stage {}", &stage);
        let _ = self
            .client
            .create_stage()
            .api_id(s!(api_id))
            .stage_name(stage.clone())
            .set_stage_variables(Some(stage_variables))
            .send()
            .await;
    }


    pub async fn create_integration(
        &self,
        api_id: &str,
        kind: &str,
        target_arn: &str

    ) -> String {

        println!("Creating integration {}", kind);
        match kind {
            "sfn" => sfn::find_or_create(
                &self.client, api_id, target_arn, &self.role
            ).await,
            "lambda" => lambda::find_or_create(
                &self.client, api_id, target_arn, &self.role
            ).await,
            _ => s!("")
        }
    }
}
