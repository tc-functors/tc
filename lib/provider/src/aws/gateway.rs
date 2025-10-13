use crate::Auth;
use aws_sdk_apigatewayv2::{
    Client,
    Error,
    types::{
        AuthorizationType,
        AuthorizerType,
        Cors,
        ProtocolType,
        builders::CorsBuilder,
    },
};
use colored::Colorize;
use kit::*;
use std::collections::HashMap;

mod eventbridge;
mod lambda;
mod sfn;
mod sqs;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

#[derive(Clone, Debug)]
pub struct Api {
    pub client: Client,
    pub name: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
    pub role: String,
    pub path: String,
    pub method: String,
    pub is_async: bool,
    pub cors: Option<Cors>,
    pub tags: HashMap<String, String>,
}

pub fn make_cors(methods: Vec<String>, origins: Vec<String>, headers: Option<Vec<String>>) -> Cors {
    let f = CorsBuilder::default();
    f.set_allow_methods(Some(methods))
        .set_allow_origins(Some(origins))
        .set_allow_headers(headers)
        .build()
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
            .set_cors_configuration(self.cors)
            .set_tags(Some(self.tags))
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

    pub async fn update(&self, api_id: &str) -> String {
        if let Some(cors) = self.cors.clone() {
            println!("Updating route {} (cors)", &self.name);
            let _ = self
                .clone()
                .client
                .update_api()
                .api_id(s!(api_id))
                .cors_configuration(cors)
                .send()
                .await
                .unwrap();
        }

        s!(api_id)
    }

    pub async fn create_or_update(&self) -> String {
        let api_id = self.find().await;
        match api_id {
            Some(id) => {
                tracing::debug!("Found API {} ({})", &self.name.green(), &id);
                self.clone().update(&id).await
            }
            _ => {
                println!("Creating route {} (gateway)", &self.name.blue());
                self.clone().create().await
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

    pub async fn update_route(
        &self,
        api_id: &str,
        route_id: &str,
        target: &str,
        authorizer: Option<String>,
    ) -> String {
        match authorizer {
            Some(auth) => {
                let res = self
                    .client
                    .update_route()
                    .api_id(s!(api_id))
                    .route_id(route_id)
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
                    .update_route()
                    .api_id(s!(api_id))
                    .route_id(route_id)
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
            Some(route_id) => {
                println!("Updating route {} ({})", &route_key.green(), &route_id);
                self.update_route(api_id, &route_id, &target, authorizer_id)
                    .await;
            }
            None => {
                println!("Creating route {}", &route_key.blue());
                self.create_route(api_id, &route_key, &target, authorizer_id)
                    .await;
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

    pub async fn find_authorizer(&self, api_id: &str, authorizer_name: &str) -> Option<String> {
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
                            if name == authorizer_name {
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

    pub async fn create_authorizer(&self, api_id: &str, name: &str, uri: &str) -> String {
        println!("Creating authorizer: {}", name.blue());
        let res = self
            .client
            .create_authorizer()
            .name(s!(name))
            .api_id(s!(api_id))
            .authorizer_type(AuthorizerType::Request)
            .authorizer_uri(s!(uri))
            .authorizer_result_ttl_in_seconds(0)
            .authorizer_payload_format_version(s!("2.0"))
            .identity_source(s!("$request.header.Authorization"))
            .send()
            .await;
        res.unwrap().authorizer_id.unwrap()
    }

    pub async fn update_authorizer(&self, id: &str, api_id: &str, uri: &str) -> String {
        let res = self
            .client
            .update_authorizer()
            .authorizer_id(s!(id))
            .api_id(s!(api_id))
            .authorizer_type(AuthorizerType::Request)
            .authorizer_uri(s!(uri))
            .authorizer_payload_format_version(s!("2.0"))
            .authorizer_result_ttl_in_seconds(0)
            .identity_source(s!("$request.header.Authorization"))
            .send()
            .await;
        res.unwrap().authorizer_id.unwrap()
    }

    pub async fn create_or_update_authorizer(&self, api_id: &str, name: &str, uri: &str) -> String {
        let maybe_authorizer_id = self.find_authorizer(api_id, name).await;
        match maybe_authorizer_id {
            Some(id) => {
                println!("Updating authorizer {}", name.green());
                self.update_authorizer(&id, api_id, uri).await
            }
            None => self.create_authorizer(api_id, name, uri).await,
        }
    }

    pub async fn delete_authorizer(&self, api_id: &str, name: &str) {
        let maybe_authorizer_id = self.find_authorizer(api_id, name).await;
        match maybe_authorizer_id {
            Some(id) => {
                println!("Deleting authorizer {} ({})", name.green(), &id);
                let _ = self
                    .client
                    .delete_authorizer()
                    .authorizer_id(s!(id))
                    .api_id(s!(api_id))
                    .authorizer_id(s!(id))
                    .send()
                    .await;
            }
            None => (),
        }
    }

    pub async fn create_stage(&self, api_id: &str) {
        let stage = self.clone().stage;
        let stage_variables = self.stage_variables.to_owned();
        tracing::debug!("Creating stage {}", &stage.green());
        let _ = self
            .client
            .create_stage()
            .api_id(s!(api_id))
            .auto_deploy(true)
            .stage_name(stage.clone())
            .set_stage_variables(Some(stage_variables))
            .send()
            .await;
    }

    pub async fn create_lambda_integration(&self, api_id: &str, target_arn: &str) -> String {
        lambda::create_or_update(&self.client, api_id, target_arn, &self.role, self.is_async).await
    }

    pub async fn create_sfn_integration(
        &self,
        api_id: &str,
        name: &str,
        request_params: HashMap<String, String>,
    ) -> String {
        sfn::find_or_create(
            &self.client,
            api_id,
            &self.role,
            request_params,
            self.is_async,
            name,
        )
        .await
    }

    pub async fn create_event_integration(
        &self,
        api_id: &str,
        name: &str,
        request_params: HashMap<String, String>,
    ) -> String {
        eventbridge::find_or_create(&self.client, api_id, &self.role, request_params, name).await
    }

    pub async fn create_sqs_integration(
        &self,
        api_id: &str,
        name: &str,
        request_params: HashMap<String, String>,
    ) -> String {
        sqs::find_or_create(&self.client, api_id, &self.role, request_params, name).await
    }

    pub async fn delete_lambda_integration(&self, api_id: &str, target_arn: &str) {
        lambda::delete(&self.client, api_id, target_arn).await
    }

    pub async fn delete_sfn_integration(&self, api_id: &str, name: &str) {
        sfn::delete(&self.client, api_id, name).await
    }

    pub async fn delete_event_integration(&self, api_id: &str, name: &str) {
        eventbridge::delete(&self.client, api_id, name).await
    }

    pub async fn delete_sqs_integration(&self, api_id: &str, name: &str) {
        sqs::delete(&self.client, api_id, name).await
    }
}

pub async fn find_api_id(client: &Client, name: &str) -> Option<String> {
    let r = client
        .get_apis()
        .max_results(String::from("1000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(apis) => {
            for api in apis.to_vec() {
                match api.name {
                    Some(n) => {
                        if n == name {
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

async fn list_apis_by_token(
    client: &Client,
    token: &str,
) -> (HashMap<String, HashMap<String, String>>, Option<String>) {
    let res = client
        .get_apis()
        .next_token(token.to_string())
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    let apis = res.items.unwrap();
    for api in apis {
        h.insert(api.name.unwrap(), api.tags.unwrap());
    }
    (h, res.next_token)
}

async fn list_apis(client: &Client) -> HashMap<String, HashMap<String, String>> {
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    let r = client.get_apis().send().await;
    match r {
        Ok(res) => {
            let mut token: Option<String> = res.next_token;

            let apis = res.items.unwrap();
            for api in apis {
                h.insert(api.name.unwrap(), api.tags.unwrap());
            }

            match token {
                Some(tk) => {
                    token = Some(tk);
                    while token.is_some() {
                        let (xs, t) = list_apis_by_token(client, &token.unwrap()).await;
                        h.extend(xs.clone());
                        token = t.clone();
                        if let Some(x) = t {
                            if x.is_empty() {
                                break;
                            }
                        }
                    }
                }
                None => (),
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
}

pub async fn find_tags(client: &Client, name: &str) -> HashMap<String, String> {
    let apis = list_apis(client).await;
    let maybe_h = apis.get(name);
    match maybe_h {
        Some(p) => p.clone(),
        None => HashMap::new(),
    }
}

pub type GatewayCors = aws_sdk_apigatewayv2::types::Cors;
