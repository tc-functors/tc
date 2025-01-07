use anyhow::Result;
use aws_sdk_sfn::config as sfn_config;
use aws_sdk_sfn::config::retry::RetryConfig;
use aws_sdk_sfn::operation::start_sync_execution::StartSyncExecutionOutput;
use aws_sdk_sfn::types::builders::{CloudWatchLogsLogGroupBuilder, LogDestinationBuilder};
use aws_sdk_sfn::types::builders::{LoggingConfigurationBuilder, TagBuilder};
use aws_sdk_sfn::types::{LogLevel, LoggingConfiguration};
use aws_sdk_sfn::types::{StateMachineStatus, StateMachineType, Tag};
use aws_sdk_sfn::{Client, Error};
use colored::Colorize;
use kit::LogUpdate;
use std::collections::HashMap;
use std::io::stdout;
use std::panic;

use super::Env;
use kit::*;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::from_conf(
        sfn_config::Builder::from(&shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(7))
            .build(),
    )
}

fn make_tag(key: String, value: String) -> Tag {
    let tb = TagBuilder::default();
    tb.key(key).value(value).build()
}

fn make_tags(kvs: HashMap<String, String>) -> Vec<Tag> {
    let mut tags: Vec<Tag> = vec![];
    for (k, v) in kvs.into_iter() {
        let tag = make_tag(k, v);
        tags.push(tag);
    }
    tags
}

fn make_log_config(log_group_arn: &str, enable: bool) -> LoggingConfiguration {
    if enable {
        let lg = CloudWatchLogsLogGroupBuilder::default();
        let group = lg.log_group_arn(log_group_arn).build();

        let ld = LogDestinationBuilder::default();
        let destination = ld.cloud_watch_logs_log_group(group).build();

        let lc = LoggingConfigurationBuilder::default();
        lc.level(LogLevel::All)
            .include_execution_data(true)
            .destinations(destination)
            .build()
    } else {
        let lc = LoggingConfigurationBuilder::default();
        lc.level(LogLevel::Off)
            .include_execution_data(false)
            .build()
    }
}

pub fn make_mode(mode: &str) -> StateMachineType {
    match mode {
        "Standard" => StateMachineType::Standard,
        "Express" => StateMachineType::Express,
        _ => StateMachineType::Standard,
    }
}

#[derive(Clone, Debug)]
pub struct StateMachine {
    pub name: String,
    pub client: Client,
    pub mode: StateMachineType,
    pub definition: String,
    pub role_arn: String,
    pub tags: HashMap<String, String>,
}

impl StateMachine {
    async fn get_state(&self, arn: &str) -> StateMachineStatus {
        let r = self
            .client
            .describe_state_machine()
            .state_machine_arn(arn)
            .send()
            .await;
        match r {
            Ok(res) => res.status.unwrap(),
            Err(_) => "NotFound".into(),
        }
    }

    async fn create(&self) {
        let name = &self.name;
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let _ = log_update.render(&format!("Creating sgn {}", name));
        let mut state: StateMachineStatus = StateMachineStatus::Deleting;

        let tags = make_tags(self.tags.clone());
        let res = self
            .clone()
            .client
            .create_state_machine()
            .name(self.name.to_owned())
            .definition(self.definition.to_owned())
            .role_arn(self.role_arn.to_owned())
            .r#type(self.mode.to_owned())
            .set_tags(Some(tags))
            .send()
            .await;

        match res {
            Ok(r) => {
                let arn = r.state_machine_arn;
                while state != StateMachineStatus::Active {
                    state = self.get_state(&arn).await;
                    let _ = log_update.render(&format!(
                        "Checking state {} ({})",
                        &name,
                        state.as_str().blue()
                    ));
                    sleep(500)
                }
                let _ = log_update.render(&format!(
                    "Checking state {} ({})",
                    &name,
                    state.as_str().green()
                ));
            }
            Err(e) => panic!("{:?}", e),
        }
    }

    async fn update(self, arn: &str) {
        let s = self.clone();
        println!("Updating sfn {}", &self.name);
        self.client
            .update_state_machine()
            .state_machine_arn(arn.to_string())
            .definition(self.definition)
            .role_arn(self.role_arn)
            .send()
            .await
            .unwrap();

        s.clone().tag_resource(arn).await
    }

    async fn tag_resource(self, arn: &str) {
        let tags = make_tags(self.tags);
        self.client
            .tag_resource()
            .resource_arn(arn.to_string())
            .set_tags(Some(tags))
            .send()
            .await
            .unwrap();
    }

    async fn exists(self, arn: &str) -> Result<bool, Error> {
        let resp = self
            .client
            .describe_state_machine()
            .state_machine_arn(arn.to_string())
            .send()
            .await;

        match resp {
            Ok(_resp) => Ok(true),
            Err(_e) => Ok(false),
        }
    }

    pub async fn create_or_update(self, arn: &str) {
        if self.clone().exists(arn).await.unwrap() {
            self.update(arn).await
        } else {
            self.create().await
        }
    }

    pub async fn delete(self, arn: &str) -> Result<(), Error> {
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let name = &self.name;
        println!("Deleting sfn {}", name);

        let mut state: StateMachineStatus = StateMachineStatus::Deleting;
        let res = self
            .client
            .delete_state_machine()
            .state_machine_arn(arn.to_string())
            .send()
            .await;

        while state == StateMachineStatus::Deleting {
            state = self.clone().get_state(name).await;
            let _ = log_update.render(&format!(
                "Checking state {} ({})",
                name,
                state.as_str().blue()
            ));
            sleep(500)
        }
        let _ = log_update.render(&format!(
            "Checking state: {} ({})",
            name,
            state.as_str().green()
        ));

        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }
}

pub async fn start_execution(client: Client, arn: &str, input: &str) -> String {
    let res = client
        .start_execution()
        .state_machine_arn(arn.to_string())
        .input(input)
        .send()
        .await;
    match res {
        Ok(r) => r.execution_arn,
        Err(_) => {
            panic::set_hook(Box::new(|_| {
                println!("Error: Failed to invoke. Check payload or sandbox");
            }));
            panic!("Failed to invoke")
        }
    }
}

pub async fn _start_sync_execution(
    client: Client,
    arn: &str,
    input: &str,
    name: Option<String>,
) -> StartSyncExecutionOutput {
    let res = client
        .start_sync_execution()
        .state_machine_arn(arn.to_string())
        .input(input)
        .set_name(name)
        .send()
        .await;
    match res {
        Ok(r) => r,
        Err(e) => panic!("error: {:?}", e),
    }
}

pub async fn list_tags(client: &Client, arn: &str) -> Result<HashMap<String, String>, Error> {
    let res = client
        .list_tags_for_resource()
        .resource_arn(arn.to_string())
        .send()
        .await;

    match res {
        Ok(r) => match r.tags {
            Some(xs) => {
                let mut h: HashMap<String, String> = HashMap::new();
                for tag in xs {
                    let k = tag.key().unwrap().to_string();
                    let v = tag.value().unwrap().to_string();
                    h.insert(k, v);
                }
                Ok(h)
            }
            _ => Ok(HashMap::new()),
        },

        Err(_) => Ok(HashMap::new()),
    }
}

pub async fn enable_logging(client: Client, arn: &str, log_arn: &str) -> Result<(), Error> {
    let log_config = make_log_config(log_arn, true);
    let res = client
        .update_state_machine()
        .state_machine_arn(arn.to_string())
        .logging_configuration(log_config)
        .send()
        .await;
    match res {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub async fn disable_logging(client: Client, arn: &str) -> Result<(), Error> {
    let log_config = make_log_config("", false);
    let res = client
        .update_state_machine()
        .state_machine_arn(arn.to_string())
        .logging_configuration(log_config)
        .send()
        .await;
    match res {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub async fn update_tags(client: &Client, arn: &str, tags: HashMap<String, String>) -> Result<()> {
    let tags = make_tags(tags);
    client
        .tag_resource()
        .resource_arn(arn.to_string())
        .set_tags(Some(tags))
        .send()
        .await?;
    Ok(())
}

pub async fn get_tag(client: &Client, arn: &str, tag: String) -> String {
    let tags = list_tags(&client, arn).await.unwrap();
    match tags.get(&tag) {
        Some(v) => v.to_string(),
        None => "".to_string(),
    }
}

pub async fn list(client: Client) -> Vec<HashMap<String, String>> {
    let res = client
        .clone()
        .list_state_machines()
        .max_results(1000)
        .send()
        .await
        .unwrap();
    let sfns = res.state_machines;
    let mut out: Vec<HashMap<String, String>> = vec![];
    for sfn in sfns {
        let mut h: HashMap<String, String> = HashMap::new();
        let arn = sfn.state_machine_arn;
        h.insert(s!("type"), sfn.r#type.as_str().to_string());
        let tags = list_tags(&client, &arn).await.unwrap();
        let namespace = tags.get("namespace");
        match namespace {
            Some(name) => {
                if !name.is_empty() {
                    h.insert(s!("version"), safe_unwrap(tags.get("version")));
                    h.insert(s!("namespace"), name.to_string());
                    h.insert(s!("sandbox"), safe_unwrap(tags.get("sandbox")));
                    h.insert(s!("updated_at"), safe_unwrap(tags.get("updated_at")));
                    out.push(h);
                }
            }
            None => (),
        }
    }
    out
}
