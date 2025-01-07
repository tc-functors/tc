use aws_sdk_apigateway::types::{
    ConnectionType, ContentHandlingStrategy, IntegrationType, Resource,
};
use aws_sdk_apigateway::{Client, Error};

use super::Env;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

#[derive(Clone, Debug)]
pub struct RestApi {
    pub client: Client,
    pub name: String,
    pub stage: String,
    pub uri: String,
    pub role: String,
    pub path: String,
    pub method: String,
    pub request_template: String,
    pub response_template: String,
}

fn unpack_resource(resource: Option<&Resource>) -> String {
    match resource {
        Some(p) => match &p.id {
            Some(id) => id.to_string(),
            _ => panic!("no root id"),
        },
        _ => panic!("no root id"),
    }
}

fn unpack_resource_id(items: Option<Vec<Resource>>) -> String {
    match items {
        Some(res) => {
            let res = res.to_vec();
            let m = res.first();
            unpack_resource(m)
        }
        _ => panic!("no root error"),
    }
}

impl RestApi {
    async fn _list(self) -> Vec<String> {
        let client = self.client;
        let r = client.get_rest_apis().send().await.unwrap();
        r.items;
        vec![]
    }

    pub async fn create(self) -> String {
        let api = self.clone();
        let r = self
            .clone()
            .client
            .create_rest_api()
            .name(api.name)
            .send()
            .await
            .unwrap();
        r.id.unwrap_or_default()
    }

    pub async fn delete(self, api_id: &str) {
        self.client
            .delete_rest_api()
            .rest_api_id(api_id)
            .send()
            .await
            .unwrap();
    }

    pub async fn find(&self) -> Option<String> {
        let r = self.client.get_rest_apis().send().await.unwrap();
        let items = r.items;
        match items {
            Some(apis) => {
                for api in apis {
                    match api.name {
                        Some(name) => {
                            if name == self.name {
                                return api.id;
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
            Some(id) => id,
            _ => self.clone().create().await,
        }
    }

    pub async fn get_root_resource(&self, api_id: &str) -> String {
        let resources = self
            .client
            .get_resources()
            .rest_api_id(api_id)
            .send()
            .await
            .unwrap();
        let items = resources.items;
        unpack_resource_id(items)
    }

    pub async fn find_resource(&self, api_id: &str, path_part: &str) -> Option<String> {
        let resources = self
            .client
            .get_resources()
            .rest_api_id(api_id)
            .send()
            .await
            .unwrap();
        match resources.items {
            Some(res) => {
                let res = res.to_vec();
                for r in res {
                    match r.path_part {
                        Some(p) => {
                            if p == path_part {
                                return r.id;
                            };
                        }
                        None => (),
                    };
                }
                return None;
            }
            None => None,
        }
    }

    pub async fn create_resource(&self, api_id: &str, parent_id: &str, path: &str) -> String {
        let res = self
            .client
            .create_resource()
            .rest_api_id(api_id)
            .parent_id(parent_id)
            .path_part(path)
            .send()
            .await;

        match res {
            Ok(r) => r.id.unwrap(),
            Err(e) => panic!("{:?}", e),
        }
    }

    pub async fn find_or_create_resource(
        &self,
        api_id: &str,
        parent_id: &str,
        path: &str,
    ) -> String {
        let res_id = self.find_resource(api_id, path).await;
        match res_id {
            Some(id) => id,
            None => self.create_resource(api_id, parent_id, path).await,
        }
    }

    pub async fn put_method(
        &self,
        api_id: &str,
        resource_id: &str,
        method: &str,
    ) -> Result<(), Error> {
        let res = self
            .client
            .put_method()
            .rest_api_id(api_id)
            .resource_id(resource_id)
            .http_method(method)
            .authorization_type("None".to_string())
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn put_integration_request(
        &self,
        api_id: &str,
        resource_id: &str,
        method: &str,
    ) -> Result<(), Error> {
        let api = self.clone();
        let res = self
            .client
            .put_integration()
            .rest_api_id(api_id)
            .resource_id(resource_id)
            .http_method(method)
            .integration_http_method("POST".to_string())
            .r#type(IntegrationType::Aws)
            .uri(api.uri)
            .credentials(api.role)
            .connection_type(ConnectionType::Internet)
            .request_templates("application/json".to_string(), api.request_template)
            .passthrough_behavior("NEVER".to_string())
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn put_integration_response(
        &self,
        api_id: &str,
        resource_id: &str,
        method: &str,
    ) -> Result<(), Error> {
        let api = self.clone();
        let res = self
            .client
            .put_integration_response()
            .rest_api_id(api_id)
            .resource_id(resource_id)
            .http_method(method)
            .status_code("200".to_string())
            .content_handling(ContentHandlingStrategy::ConvertToText)
            .response_templates("application/json".to_string(), api.response_template)
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn put_method_response(
        &self,
        api_id: &str,
        resource_id: &str,
        method: &str,
    ) -> Result<(), Error> {
        let res = self
            .client
            .put_method_response()
            .rest_api_id(api_id)
            .resource_id(resource_id)
            .http_method(method)
            .status_code("200".to_string())
            .response_models("application/json".to_string(), "Empty".to_string())
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn create_deployment(&self, api_id: &str, stage: &str) {
        self.client
            .create_deployment()
            .rest_api_id(api_id)
            .stage_name(stage)
            .send()
            .await
            .unwrap();
    }
}
