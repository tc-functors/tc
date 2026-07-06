use aws_sdk_lambdamicrovms::Client;
use aws_sdk_lambdamicrovms::types::CodeArtifact;
use aws_sdk_lambdamicrovms::types::MicrovmImageState;
use aws_sdk_lambdamicrovms::types::MicrovmState;
use aws_sdk_lambdamicrovms::types::PortSpecification;
use crate::Auth;

use colored::Colorize;
use kit::{
    LogUpdate,
    *,
};
use std::{
    io::{
        stdout,
    },
};


pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

fn pp_status(status: &MicrovmImageState) -> String {
    match status {
        MicrovmImageState::Created => s!("ok"),
        MicrovmImageState::CreateFailed => s!("failed"),
        _ => s!("pending")
    }
}

pub struct MicroVmImage {
    pub name: String,
    pub base_image_arn: String,
    pub build_role_arn: String,
    pub uri: String,
}

impl MicroVmImage {

    async fn find(&self, client: &Client) -> Option<String> {
        let res = client
            .list_microvm_images()
            .name_filter(self.name.clone())
            .send()
            .await
            .unwrap();
        let xs = res.items.to_vec();
        for x in xs {
            if x.name == self.name {
                return Some(x.image_arn)
            }
        }
        None
    }

    async fn create(&self, client: &Client, idempotency_token: &str) -> String {
        let res = client
            .create_microvm_image()
            .name(self.name.clone())
            .base_image_arn(self.base_image_arn.clone())
            .build_role_arn(self.build_role_arn.clone())
            .code_artifact(CodeArtifact::Uri(self.uri.clone()))
            .client_token(idempotency_token)
            .send()
            .await
            .unwrap();
        let image_id = res.image_arn;
        self.wait(client, &image_id).await;
        image_id

    }

    pub async fn find_or_create(&self, client: &Client, idempotency_token: &str) -> String {
        let maybe_id = self.find(client).await;
        if let Some(image_arn) = maybe_id {
            println!("Found image {}", &image_arn);
            image_arn
        } else {
            println!("Creating image {}", self.name);
            self.create(client, idempotency_token).await
        }
    }

    pub async fn update(&self, client: &Client, image_id: &str){
        let _ = client
            .update_microvm_image()
            .image_identifier(image_id)
            .base_image_arn(self.base_image_arn.clone())
            .code_artifact(CodeArtifact::Uri(self.uri.clone()))
            .send()
            .await
            .unwrap();
    }

    async fn get_state(&self, client: &Client, id: &str) -> MicrovmImageState {
        let res = client
            .get_microvm_image()
            .image_identifier(id)
            .send()
            .await
            .unwrap();

        res.state
    }

    async fn wait(&self, client: &Client, id: &str) {
        let mut state: MicrovmImageState = MicrovmImageState::Creating;
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        while state != MicrovmImageState::Created {
            state = self.get_state(client, id).await;
            let _ = log_update.render(&format!("{} state {}", self.name, pp_status(&state).blue()));
            sleep(5000)
        }
    }


    pub async fn delete(&self, client: &Client, id: &str) {
        let _ = client
            .delete_microvm_image()
            .image_identifier(id)
            .send()
            .await
            .unwrap();
    }
}

pub struct MicroVm {
    pub image_id: String,
    pub role: String,
    pub ingress_network_connectors: String,
    pub egress_network_connectors: String,
    pub idle_policy: String
}

pub struct RunInfo {
    pub microvm_id: String,
    pub endpoint: String,
    pub state: MicrovmState,
    pub state_reason: Option<String>,
}

impl MicroVm {

    pub async fn run(&self, client: &Client) -> RunInfo {
        let res = client
            .run_microvm()
            .image_identifier(&self.image_id)
            .execution_role_arn(&self.role)
            .client_token(&self.image_id)
            .send()
            .await
            .unwrap();

        RunInfo {
            microvm_id: res.microvm_id,
            state: res.state,
            endpoint: res.endpoint,
            state_reason: res.state_reason
        }
    }
}

pub async fn get_token(client: &Client, microvm_id: &str, exp: i32) -> Option<String> {
    let res = client
        .create_microvm_auth_token()
        .microvm_identifier(microvm_id)
        .expiration_in_minutes(exp)
        .allowed_ports(PortSpecification::AllPorts)
        .send()
        .await
        .unwrap();

    res.auth_token.get("X-aws-proxy-auth").cloned()
}

pub async fn terminate(client: &Client, microvm_id: &str) {
    let _ = client
        .terminate_microvm()
        .microvm_identifier(microvm_id)
        .send()
        .await
        .unwrap();
}


pub async fn suspend(client: &Client, microvm_id: &str) {
    let _ = client
        .suspend_microvm()
        .microvm_identifier(microvm_id)
        .send()
        .await
        .unwrap();
}

pub async fn resume(client: &Client, microvm_id: &str) {
    let _ = client
        .resume_microvm()
        .microvm_identifier(microvm_id)
        .send()
        .await
        .unwrap();
}

pub async fn find_image(client: &Client, image_name: &str) -> Option<String> {
    let res = client
        .list_microvm_images()
        .name_filter(image_name)
        .send()
        .await
        .unwrap();
    let xs = res.items.to_vec();
    for x in xs {
        if x.name == image_name {
            return Some(x.image_arn)
        }
    }
    None
}

pub async fn find(client: &Client, image_name: &str) -> Option<String> {
    let maybe_image_id = find_image(client, image_name).await;
    if let Some(image_id) = maybe_image_id {
        let res = client
            .list_microvms()
            .image_identifier(image_id)
            .send()
            .await
            .unwrap();
        let xs = res.items.to_vec();
        match xs.first() {
            Some(x) => Some(x.microvm_id.clone()),
            None => None
        }
    } else {
        None
    }
}


pub async fn find_by_image_id(client: &Client, image_id: &str) -> Option<String> {
    let res = client
        .list_microvms()
        .image_identifier(image_id)
        .send()
        .await
        .unwrap();
    let xs = res.items.to_vec();
    match xs.first() {
        Some(x) => Some(x.microvm_id.clone()),
        None => None
    }
}

pub async fn get_microvm(client: &Client, microvm_id: &str) -> RunInfo {
        let res = client
            .get_microvm()
            .microvm_identifier(microvm_id)
            .send()
            .await
            .unwrap();

        RunInfo {
            microvm_id: res.microvm_id,
            endpoint: res.endpoint,
            state: res.state,
            state_reason: res.state_reason
        }
}

pub async fn is_suspended(client: &Client, microvm_id: &str) -> bool {
    let res = client
        .get_microvm()
        .microvm_identifier(microvm_id)
        .send()
        .await
        .unwrap();
    res.state == MicrovmState::Suspended
}

pub type MicroVmClient = Client;
