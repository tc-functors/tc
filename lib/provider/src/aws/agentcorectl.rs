use crate::Auth;
use aws_sdk_bedrockagentcorecontrol::{
    Client,
};
use aws_sdk_bedrockagentcorecontrol::types::NetworkMode;
use aws_sdk_bedrockagentcorecontrol::types::S3Location;
use aws_sdk_bedrockagentcorecontrol::types::Code;
use aws_sdk_bedrockagentcorecontrol::types::CodeConfiguration;
use aws_sdk_bedrockagentcorecontrol::types::NetworkConfiguration;
use aws_sdk_bedrockagentcorecontrol::types::AgentRuntimeArtifact;
use aws_sdk_bedrockagentcorecontrol::types::AgentManagedRuntimeType;
use aws_sdk_bedrockagentcorecontrol::types::builders::{
    CodeConfigurationBuilder,
    NetworkConfigurationBuilder,
    S3LocationBuilder
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

fn make_s3_location(bucket: &str, prefix: &str) -> S3Location {
    let v = S3LocationBuilder::default();
    v.bucket(bucket).prefix(prefix).build().unwrap()
}

fn make_code_config(bucket: &str, prefix: &str, langr: &str, handler: &str) -> CodeConfiguration {
    let runtime = match langr {
        "python3.12" => AgentManagedRuntimeType::Python312,
        "python3.13" => AgentManagedRuntimeType::Python313,
        "python3.14" => AgentManagedRuntimeType::Python314,
        _ => AgentManagedRuntimeType::Python312
    };

    let v = CodeConfigurationBuilder::default();
    let s3_location = make_s3_location(bucket, prefix);
    v.code(Code::S3(s3_location))
        .runtime(runtime)
        .entry_point(handler)
        .build()
        .unwrap()
}

fn make_network() -> NetworkConfiguration {
    let v = NetworkConfigurationBuilder::default();
    v.network_mode(NetworkMode::Public).build().unwrap()
}


pub struct Runtime {
    pub name: String,
    pub langr: String,
    pub bucket: String,
    pub prefix: String,
    pub role: String,
    pub handler: String
}

impl Runtime {

    async fn find(&self, client: &Client) -> Option<String> {
        let res = client
            .list_agent_runtimes()
            .send()
            .await
            .unwrap();
        let xs = res.agent_runtimes.to_vec();
        for x in xs {
            if x.agent_runtime_name == self.name {
                return Some(x.agent_runtime_id)
            }
        }
        None
    }

    async fn update(&self, client: &Client, id: &str) -> String {
        let code = make_code_config(&self.bucket, &self.prefix, &self.langr, &self.handler);
        let network = make_network();
        let res = client
            .update_agent_runtime()
            .agent_runtime_id(id)
            .agent_runtime_artifact(AgentRuntimeArtifact::CodeConfiguration(code))
            .role_arn(self.role.clone())
            .network_configuration(network)
            .send()
            .await
            .unwrap();
        res.agent_runtime_arn
    }

    async fn create(&self, client: &Client) -> String {
        let code = make_code_config(&self.bucket, &self.prefix, &self.langr, &self.handler);
        let network = make_network();
        let res = client
            .create_agent_runtime()
            .agent_runtime_name(self.name.clone())
            .agent_runtime_artifact(AgentRuntimeArtifact::CodeConfiguration(code))
            .role_arn(self.role.clone())
            .network_configuration(network)
            .send()
            .await
            .unwrap();
        res.agent_runtime_arn
    }

    pub async fn create_or_update(&self, client: &Client) -> String {
        let maybe_id = self.find(client).await;
        match maybe_id {
            Some(id) => self.update(client, &id).await,
            None => self.create(client).await
        }
    }
}

pub async fn find(client: &Client, name: &str) -> Option<String> {
    let res = client
        .list_agent_runtimes()
        .send()
        .await
        .unwrap();
    let xs = res.agent_runtimes.to_vec();
    for x in xs {
        if x.agent_runtime_name == name {
            return Some(x.agent_runtime_id)
        }
    }
    None
}

pub async fn delete(client: &Client, id: &str) {
    let _ = client
        .delete_agent_runtime()
        .agent_runtime_id(id)
        .send()
        .await
        .unwrap();
}
