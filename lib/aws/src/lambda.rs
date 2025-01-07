use super::Env;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::config as lambda_config;
use aws_sdk_lambda::config::retry::RetryConfig;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::builders::{
    DeadLetterConfigBuilder, EnvironmentBuilder, FileSystemConfigBuilder, FunctionCodeBuilder,
    VpcConfigBuilder,
};
use aws_sdk_lambda::types::Architecture;
use aws_sdk_lambda::types::DeadLetterConfig;
use aws_sdk_lambda::types::LastUpdateStatus;
use aws_sdk_lambda::types::{Environment, FunctionCode, PackageType, Runtime, State};
use aws_sdk_lambda::types::{FileSystemConfig, VpcConfig};
use aws_sdk_lambda::types::{InvocationType, LogType, LoggingConfig};
use aws_sdk_lambda::{Client, Error};
use base64::{engine::general_purpose, Engine as _};
use colored::Colorize;
use kit::LogUpdate;
use kit::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::stdout;
use std::io::BufReader;
use std::io::Read;
use std::panic;

fn pp_state(state: &State) -> String {
    match state {
        State::Active => s!("ok"),
        State::Failed => s!("failed"),
        State::Pending => s!("pending"),
        State::Inactive => s!("inactive"),
        &_ => todo!(),
    }
}

fn pp_status(status: &LastUpdateStatus) -> String {
    match status {
        LastUpdateStatus::Successful => s!("ok"),
        LastUpdateStatus::Failed => s!("failed"),
        LastUpdateStatus::InProgress => s!("pending"),
        &_ => todo!(),
    }
}

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::from_conf(
        lambda_config::Builder::from(&shared_config)
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

pub fn make_blob_from_str(payload: &str) -> Blob {
    let buffer = payload.as_bytes();
    Blob::new(buffer)
}

fn make_blob(payload_file: &str) -> Blob {
    if file_exists(payload_file) {
        let f = File::open(payload_file).unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        Blob::new(buffer)
    } else {
        make_blob_from_str("test")
    }
}

pub fn make_fs_config(efs_ap_arn: &str, mount_point: &str) -> FileSystemConfig {
    let f = FileSystemConfigBuilder::default();
    f.arn(efs_ap_arn)
        .local_mount_path(mount_point)
        .build()
        .unwrap()
}

pub fn make_vpc_config(subnets: Vec<String>, sgs: Vec<String>) -> VpcConfig {
    let v = VpcConfigBuilder::default();
    v.set_subnet_ids(Some(subnets))
        .set_security_group_ids(Some(sgs))
        .build()
}

pub fn make_code(package_type: &str, path: &str) -> (String, Blob, FunctionCode) {
    match package_type {
        "zip" => {
            let blob = make_blob(path);
            let f = FunctionCodeBuilder::default();
            let code = f.zip_file(blob.clone()).build();
            let size: f64 = blob.clone().into_inner().len() as f64;
            let hsize = file_size_human(size);
            (hsize, blob, code)
        }
        "image" => {
            let f = FunctionCodeBuilder::default();
            let code = f.image_uri(path).build();
            let blob = make_blob_from_str("default");
            (s!("image"), blob, code)
        }
        _ => todo!(),
    }
}

pub fn make_environment(vars: HashMap<String, String>) -> Environment {
    let e = EnvironmentBuilder::default();
    e.set_variables(Some(vars)).build()
}

pub fn make_runtime(lang: &str) -> Runtime {
    match lang {
        "java11" => Runtime::Java11,
        "ruby2.7" => Runtime::Ruby27,
        "go" => "provided.al2023".into(),
        "python3.7" => Runtime::Python37,
        "python3.8" => Runtime::Python38,
        "python3.9" => Runtime::Python39,
        "python3.10" => Runtime::Python310,
        "python3.11" => Runtime::Python311,
        "python3.12" => Runtime::Python312,
        "provided" => Runtime::Provided,
        "providedal2" => Runtime::Providedal2,
        "janet" => "provided.al2023".into(),
        "rust" => "provided.al2023".into(),
        "ruby3.2" => "ruby3.2".into(),
        _ => Runtime::Provided,
    }
}

pub fn make_arch(lang: &str) -> Architecture {
    // hack
    if lang == "go" {
        Architecture::Arm64
    } else {
        Architecture::X8664
    }
}

pub fn make_package_type(package_type: &str) -> PackageType {
    match package_type {
        "zip" => PackageType::Zip,
        "image" => PackageType::Image,
        _ => PackageType::Zip,
    }
}

fn print_logs(log_result: Option<String>, payload: Option<Blob>) {
    match log_result {
        Some(x) => {
            let bytes = general_purpose::STANDARD.decode(x).unwrap();
            let logs = String::from_utf8_lossy(&bytes);
            let xs = logs.split("\n").collect::<Vec<&str>>();
            for log in xs {
                if log.contains("error") || log.contains("ERROR") {
                    println!("{}", log);
                } else {
                    println!("{}", log);
                }
            }
        }
        _ => {
            println!("");
        }
    };

    match payload {
        Some(p) => {
            println!("response: {}", String::from_utf8_lossy(&p.into_inner()));
        }
        _ => {
            println!("");
        }
    };
}

#[derive(Clone, Debug)]
pub struct Function {
    pub client: Client,
    pub name: String,
    pub actual_name: String,
    pub description: Option<String>,
    pub role: String,
    pub code_size: String,
    pub code: FunctionCode,
    pub blob: Blob,
    pub runtime: Runtime,
    pub uri: String,
    pub handler: String,
    pub timeout: i32,
    pub memory_size: i32,
    pub package_type: PackageType,
    pub environment: Environment,
    pub architecture: Architecture,
    pub tags: HashMap<String, String>,
    pub layers: Vec<String>,
    pub vpc_config: Option<VpcConfig>,
    pub filesystem_config: Option<Vec<FileSystemConfig>>,
    pub logging_config: Option<LoggingConfig>,
}

impl Function {
    async fn find_arn(self) -> Option<String> {
        let f = self.clone();
        let r = self
            .client
            .get_function_configuration()
            .function_name(f.name)
            .send()
            .await;
        match r {
            Ok(res) => res.function_arn,
            Err(_e) => None,
        }
    }

    async fn get_state(&self, name: &str) -> State {
        let r = self
            .client
            .get_function_configuration()
            .function_name(s!(name))
            .send()
            .await;
        match r {
            Ok(res) => res.state.unwrap(),
            Err(_) => State::Failed,
        }
    }

    async fn get_update_status(&self, name: &str) -> LastUpdateStatus {
        let r = self
            .client
            .get_function_configuration()
            .function_name(s!(name))
            .send()
            .await;
        match r {
            Ok(res) => res.last_update_status.unwrap(),
            Err(_) => LastUpdateStatus::InProgress,
        }
    }

    async fn wait(self, name: &str) {
        let mut state: LastUpdateStatus = LastUpdateStatus::InProgress;
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        while state != LastUpdateStatus::Successful {
            state = self.clone().get_update_status(name).await;
            let _ = log_update.render(&format!("{} state {}", name, pp_status(&state).blue()));
            sleep(1000)
        }
        let _ = log_update.render(&format!("{} state {}", name, pp_status(&state).green()));
    }

    pub async fn create(self) -> Result<String> {
        let f = self.clone();
        let mut log_update = LogUpdate::new(stdout()).unwrap();

        let name = if kit::trace() {
            &f.name
        } else {
            &f.actual_name
        };

        let _ = log_update.render(&format!(
            "Creating function {} ({})",
            name,
            &f.code_size.cyan()
        ));
        let mut state: State = State::Inactive;

        let res = self
            .client
            .create_function()
            .function_name(f.name.to_owned())
            .set_description(f.description)
            .runtime(f.runtime)
            .role(f.role)
            .handler(f.handler)
            .code(f.code)
            .environment(f.environment)
            .memory_size(f.memory_size)
            .timeout(f.timeout)
            .set_layers(Some(f.layers))
            .package_type(f.package_type)
            .set_tags(Some(f.tags))
            .set_vpc_config(f.vpc_config)
            .architectures(f.architecture)
            .set_file_system_configs(f.filesystem_config)
            .send()
            .await?;

        while state != State::Active {
            state = self.clone().get_state(&f.name).await;
            let _ = log_update.render(&format!(
                "Checking state {} ({})",
                name,
                pp_state(&state).blue()
            ));
            sleep(800)
        }
        let _ = log_update.render(&format!(
            "Checking state {} ({})",
            name,
            pp_state(&state).green()
        ));

        Ok(res.function_arn.unwrap_or_default())
    }

    pub async fn update_function(self, arn: &str) -> Result<String, Error> {
        let f = self.clone();
        let name = if kit::trace() {
            &f.name
        } else {
            &f.actual_name
        };

        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let _ = log_update.render(&format!(
            "Updating function {} ({})",
            name,
            &f.code_size.cyan()
        ));
        let mut state: LastUpdateStatus = LastUpdateStatus::InProgress;
        while state != LastUpdateStatus::Successful {
            state = self.clone().get_update_status(&f.name).await;
            sleep(800)
        }

        let res = self
            .client
            .update_function_configuration()
            .function_name(arn)
            .set_layers(Some(f.layers))
            .role(f.role)
            .runtime(f.runtime)
            .handler(f.handler)
            .environment(f.environment)
            .timeout(f.timeout)
            .memory_size(f.memory_size)
            .set_vpc_config(f.vpc_config)
            .set_file_system_configs(f.filesystem_config)
            .send()
            .await;

        while state != LastUpdateStatus::Successful {
            state = self.clone().get_update_status(&f.name).await;
            sleep(800)
        }
        match res {
            Ok(r) => Ok(r.function_arn.unwrap_or_default()),
            Err(e) => panic!("{:?}", e),
        }
    }

    pub async fn update_code(self, arn: &str) -> Result<String> {
        let f = self.clone();

        let name = if kit::trace() {
            &f.name
        } else {
            &f.actual_name
        };

        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let _ = log_update.render(&format!("Updating code {} ({})", name, &f.code_size.cyan()));
        let mut state: LastUpdateStatus = LastUpdateStatus::InProgress;

        let res = self
            .client
            .update_function_code()
            .function_name(arn)
            .zip_file(f.blob)
            .publish(true)
            .send()
            .await?;

        while state != LastUpdateStatus::Successful {
            state = self.clone().get_update_status(&f.name).await;
            let _ = log_update.render(&format!(
                "Checking state {} ({})",
                name,
                pp_status(&state).blue()
            ));
            sleep(500)
        }
        let _ = log_update.render(&format!(
            "Checking state {} ({})",
            name,
            pp_status(&state).green()
        ));
        Ok(res.function_arn.unwrap_or_default())
    }

    pub async fn update_layers(self, arn: &str) -> Result<String> {
        let f = self.clone();
        println!("Updating layer {} {:?}", &f.name, &f.layers);
        let r = self
            .client
            .update_function_configuration()
            .function_name(arn)
            .set_layers(Some(f.layers))
            .send()
            .await
            .unwrap();
        self.wait(&f.name).await;
        Ok(r.function_arn.unwrap_or_default())
    }

    pub async fn update_vars(self) -> Result<String> {
        let f = self.clone();
        println!("Updating vars {}", &f.name.blue());
        let r = self
            .client
            .update_function_configuration()
            .function_name(f.name.to_owned())
            .memory_size(f.memory_size)
            .timeout(f.timeout)
            .environment(f.environment)
            .handler(f.handler)
            .send()
            .await?;
        self.wait(&f.name).await;
        Ok(r.function_arn.unwrap_or_default())
    }

    pub async fn update_image_vars(self) -> String {
        let f = self.clone();
        println!("Updating vars {}", &f.name);
        let r = self
            .client
            .update_function_configuration()
            .function_name(f.name)
            .memory_size(f.memory_size)
            .timeout(f.timeout)
            .environment(f.environment)
            .send()
            .await
            .unwrap();
        r.function_arn.unwrap_or_default()
    }

    pub async fn create_or_update(self) -> String {
        let res = self.clone().find_arn().await;
        let arn = match res {
            Some(arn) => {
                self.clone().update_code(&arn).await.unwrap();
                self.clone().update_function(&arn).await.unwrap()
            }
            None => self.create().await.unwrap(),
        };
        arn
    }

    pub async fn delete(self) -> Result<()> {
        let f = self.clone();
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let name = if kit::trace() {
            &f.name
        } else {
            &f.actual_name
        };
        let _ = log_update.render(&format!("Deleting function {}", name));
        let mut state: State = State::Active;

        let res = f.clone().find_arn().await;

        match res {
            Some(_) => {
                let _ = self
                    .client
                    .delete_function()
                    .function_name(f.name.to_owned())
                    .send()
                    .await?;

                while state == State::Active || state != State::Failed {
                    state = self.clone().get_state(&f.name).await;

                    if state != State::Failed {
                        let _ = log_update.render(&format!(
                            "Checking state {} ({})",
                            name,
                            pp_state(&state).blue()
                        ));
                    }
                    sleep(500)
                }
                if state == State::Failed {
                    let _ =
                        log_update.render(&format!("Checking state {} ({})", name, "ok".green()));
                }
                Ok(())
            }
            None => {
                let _ = log_update.render(&format!(
                    "Checking state {} ({})",
                    name,
                    "does not exist".red()
                ));
                Ok(())
            }
        }
    }

    pub async fn update_concurrency(self, n: i32) {
        let f = self.clone();
        println!("Setting provisioned concurrency {} {}", &f.name, n);
        let res = self
            .client
            .put_provisioned_concurrency_config()
            .function_name(f.name)
            .qualifier(s!("1"))
            .provisioned_concurrent_executions(n)
            .send()
            .await
            .unwrap();
        println!("{:?}", res);
    }
}

pub async fn add_permission(
    client: Client,
    name: &str,
    principal: &str,
    source_arn: &str,
    statement_id: &str,
) -> Result<()> {
    client
        .add_permission()
        .function_name(name.to_string())
        .statement_id(s!(statement_id))
        .action(s!("lambda:InvokeFunction"))
        .principal(principal.to_string())
        .source_arn(source_arn.to_string())
        .send()
        .await?;
    Ok(())
}

pub async fn add_permission_basic(
    client: Client,
    name: &str,
    principal: &str,
    statement_id: &str,
) -> Result<()> {
    client
        .add_permission()
        .function_name(name.to_string())
        .statement_id(s!(statement_id))
        .action("lambda:InvokeFunction".to_string())
        .principal(principal.to_string())
        .send()
        .await?;
    Ok(())
}

pub async fn invoke(client: Client, service: &str, payload: &str) -> Result<()> {
    let blob = make_blob_from_str(payload);
    let r = client
        .invoke()
        .function_name(service)
        .payload(blob)
        .invocation_type(InvocationType::RequestResponse)
        .log_type(LogType::Tail)
        .send()
        .await?;

    print_logs(r.log_result, r.payload);
    Ok(())
}

pub async fn invokeq(client: Client, service: &str, payload: &str) -> Option<String> {
    let blob = make_blob_from_str(payload);
    let r = client
        .invoke()
        .function_name(service)
        .payload(blob)
        .invocation_type(InvocationType::RequestResponse)
        .log_type(LogType::None)
        .send()
        .await
        .unwrap();
    let payload = r.payload;
    match payload {
        Some(p) => Some(String::from_utf8_lossy(&p.into_inner()).to_string()),
        None => None,
    }
}

pub async fn update_tags(client: Client, name: &str, arn: &str, tags: HashMap<String, String>) {
    println!("Updating tags {}", name);
    client
        .tag_resource()
        .resource(arn)
        .set_tags(Some(tags))
        .send()
        .await
        .unwrap();
}

pub async fn list_tags(client: Client, arn: &str) -> Result<HashMap<String, String>, Error> {
    let res = client.list_tags().resource(arn).send().await;

    match res {
        Ok(r) => {
            let maybe_tags = r.tags();
            match maybe_tags {
                Some(tags) => Ok(tags.clone()),
                None => Ok(HashMap::new()),
            }
        }
        Err(_) => Ok(HashMap::new()),
    }
}

pub async fn find_function_layers(
    client: &Client,
    name: &str,
) -> Result<HashMap<String, i64>, Error> {
    let res = client
        .get_function_configuration()
        .function_name(name)
        .send()
        .await;

    let mut h: HashMap<String, i64> = HashMap::new();

    match res {
        Ok(r) => {
            match r.layers {
                Some(xs) => {
                    for x in xs {
                        h.insert(x.arn.unwrap(), x.code_size);
                    }
                }
                None => (),
            }
            Ok(h)
        }
        Err(_) => Ok(HashMap::new()),
    }
}

pub struct Config {
    pub code_size: i64,
    pub timeout: i32,
    pub mem_size: i32,
    pub revision: String,
}

pub async fn find_config(client: &Client, name: &str) -> Option<Config> {
    let r = client
        .get_function_configuration()
        .function_name(s!(name))
        .send()
        .await;
    match r {
        Ok(res) => {
            let cfg = Config {
                code_size: res.code_size,
                timeout: res.timeout.unwrap(),
                mem_size: res.memory_size.unwrap(),
                revision: split_last(&res.revision_id.unwrap(), "-"),
            };
            Some(cfg)
        }
        Err(_e) => None,
    }
}

pub async fn get_code(client: &Client, name: &str) -> Option<String> {
    let r = client.get_function().function_name(s!(name)).send().await;

    match r {
        Ok(res) => match res.code {
            Some(code) => code.location,
            None => None,
        },
        Err(e) => panic!("{}", e),
    }
}

pub fn make_deadletter(sqs_arn: &str) -> DeadLetterConfig {
    let v = DeadLetterConfigBuilder::default();
    v.set_target_arn(Some(s!(sqs_arn))).build()
}

pub async fn update_dlq(client: &Client, name: &str, sqs_arn: &str) {
    let config = make_deadletter(sqs_arn);
    let _ = client
        .update_function_configuration()
        .function_name(s!(name))
        .dead_letter_config(config)
        .send()
        .await;
}

async fn find_event_source(client: &Client, name: &str, source_arn: &str) -> Option<String> {
    let r = client
        .list_event_source_mappings()
        .event_source_arn(String::from(source_arn))
        .function_name(String::from(name))
        .send()
        .await;
    let mappings = match r {
        Ok(res) => {
            if let Some(p) = res.event_source_mappings {
                p
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    };
    if mappings.len() > 0 {
        mappings.first().unwrap().uuid.to_owned()
    } else {
        None
    }
}

pub async fn create_event_source(client: &Client, name: &str, source_arn: &str) {
    let maybe_es = find_event_source(client, name, source_arn).await;
    match maybe_es {
        Some(_) => println!("Event source mapping exists, skipping"),
        None => {
            let r = client
                .create_event_source_mapping()
                .function_name(s!(name))
                .enabled(true)
                .event_source_arn(s!(source_arn))
                .batch_size(1)
                .send()
                .await;

            match r {
                Ok(_) => (),
                Err(_) => panic!("{:?}", r),
            }
        }
    }
}

pub async fn delete_event_source(client: &Client, name: &str, source_arn: &str) {
    let maybe_es = find_event_source(client, name, source_arn).await;
    match maybe_es {
        Some(uuid) => {
            let _ = client.delete_event_source_mapping().uuid(uuid).send().await;
        }
        None => (),
    }
}

pub async fn update_event_invoke_config(client: &Client, name: &str) {
    let res = client
        .put_function_event_invoke_config()
        .function_name(s!(name))
        .maximum_retry_attempts(2)
        .maximum_event_age_in_seconds(60)
        .send()
        .await;
    match res {
        Ok(_) => (),
        Err(_) => panic!("{:?}", res),
    }
}

pub type LambdaClient = Client;
