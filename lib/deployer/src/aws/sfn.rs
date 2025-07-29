use anyhow::Result;
use authorizer::Auth;
use aws_sdk_sfn::{
    Client,
    Error,
    config as sfn_config,
    config::retry::RetryConfig,
    types::{
        LogLevel,
        LoggingConfiguration,
        StateMachineStatus,
        StateMachineType,
        Tag,
        TracingConfiguration,
        builders::{
            CloudWatchLogsLogGroupBuilder,
            LogDestinationBuilder,
            LoggingConfigurationBuilder,
            TagBuilder,
            TracingConfigurationBuilder,
        },
    },
};
use colored::Colorize;
use kit::{
    LogUpdate,
    *,
};
use std::{
    collections::HashMap,
    io::stdout,
    panic,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        sfn_config::Builder::from(shared_config)
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

fn make_tracing_config() -> TracingConfiguration {
    let tc = TracingConfigurationBuilder::default();
    tc.enabled(true).build()
}

fn make_log_config(log_group_arn: &str, include_data: bool) -> LoggingConfiguration {
    let lg = CloudWatchLogsLogGroupBuilder::default();
    let group = lg.log_group_arn(log_group_arn).build();

    let ld = LogDestinationBuilder::default();
    let destination = ld.cloud_watch_logs_log_group(group).build();

    let log_level = match std::env::var("TC_SFN_LOG_LEVEL") {
        Ok(v) => match v.as_ref() {
            "ALL" => LogLevel::All,
            "ERROR" => LogLevel::Error,
            "FATAL" => LogLevel::Fatal,
            "OFF" => LogLevel::Off,
            _ => LogLevel::All,
        },
        Err(_) => LogLevel::All,
    };

    let lc = LoggingConfigurationBuilder::default();
    lc.level(log_level)
        .include_execution_data(include_data)
        .destinations(destination)
        .build()
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
        let tracing = make_tracing_config();

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
            .tracing_configuration(tracing)
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
        let tracing = make_tracing_config();

        self.client
            .update_state_machine()
            .state_machine_arn(arn.to_string())
            .role_arn(self.role_arn)
            .send()
            .await
            .unwrap();


        self.client
            .update_state_machine()
            .state_machine_arn(arn.to_string())
            .definition(self.definition)
            .tracing_configuration(tracing)
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

pub async fn enable_logging(
    client: Client,
    arn: &str,
    log_arn: &str,
    include_data: bool,
) -> Result<(), Error> {
    let log_config = make_log_config(log_arn, include_data);
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
