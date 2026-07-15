use crate::Auth;
use anyhow::Result;
use aws_sdk_lambda::{
    Client,
    config,
    config::retry::{RetryConfig, RetryMode},
    Error,
    primitives::Blob,
    types::{
        Architecture,
        DeadLetterConfig,
        DestinationConfig,
        Environment,
        FileSystemConfig,
        FunctionCode,
        InvocationType,
        LastUpdateStatus,
        LogType,
        LoggingConfig,
        PackageType,
        Runtime,
        SnapStart,
        SnapStartApplyOn,
        State,
        UpdateRuntimeOn,
        VpcConfig,
        builders::{
            DeadLetterConfigBuilder,
            DestinationConfigBuilder,
            EnvironmentBuilder,
            FileSystemConfigBuilder,
            FunctionCodeBuilder,
            OnSuccessBuilder,
            SnapStartBuilder,
            VpcConfigBuilder,
        },
    },
};
use base64::{
    Engine as _,
    engine::general_purpose,
};
use colored::Colorize;
use kit::{
    LogUpdate,
    *,
};
use std::{
    collections::HashMap,
    fs::File,
    io::{
        BufReader,
        Read,
        stdout,
    },
    panic,
};
use super::constants;

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

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        config::Builder::from(shared_config)
            .behavior_version(constants::behavior_version())
            .timeout_config(constants::timeout_config())
            .retry_config(
                RetryConfig::standard()
                    .with_retry_mode(RetryMode::Adaptive)
                    .with_max_attempts(constants::MAX_ATTEMPTS)
                    .with_initial_backoff(constants::INITIAL_BACKOFF)
                    .with_max_backoff(constants::MAX_BACKOFF)
            )
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
        "image" | "oci" => {
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

pub fn make_snapstart(enable_snap: bool) -> Option<SnapStart> {
    if enable_snap {
        let e = SnapStartBuilder::default();
        Some(
            e.set_apply_on(Some(SnapStartApplyOn::PublishedVersions))
                .build(),
        )
    } else {
        None
    }
}

pub fn make_runtime(lang: &str) -> Runtime {
    match lang {
        "java11" => Runtime::Java11,
        "go" => "provided.al2023".into(),
        "python3.7" => Runtime::Python37,
        "python3.8" => Runtime::Python38,
        "python3.9" => Runtime::Python39,
        "python3.10" => Runtime::Python310,
        "python3.11" => Runtime::Python311,
        "python3.12" => Runtime::Python312,
        "python3.13" => Runtime::Python313,
        "provided" => Runtime::Provided,
        "providedal2" => Runtime::Providedal2,
        "node22" => Runtime::Nodejs22x,
        "node20" => Runtime::Nodejs20x,
        "janet" => "provided.al2023".into(),
        "rust" => "provided.al2023".into(),
        "ruby2.7" => Runtime::Ruby27,
        "ruby3.2" => "ruby3.2".into(),
        "ruby3.4" => "ruby3.4".into(),
        "python3.14" => "python3.14".into(),
        _ => Runtime::Provided,
    }
}

pub fn make_arch(arch: &str) -> Architecture {
    match arch {
        "x8664" => Architecture::X8664,
        "arm64" => Architecture::Arm64,
        _ => Architecture::X8664,
    }
}

pub fn make_package_type(package_type: &str) -> PackageType {
    match package_type {
        "zip" => PackageType::Zip,
        "image" | "oci" => PackageType::Image,
        _ => PackageType::Zip,
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub actual_name: String,
    pub description: Option<String>,
    pub role: String,
    pub code_size: String,
    pub code: FunctionCode,
    pub blob: Blob,
    pub runtime: Option<Runtime>,
    pub uri: String,
    pub handler: Option<String>,
    pub timeout: i32,
    pub memory_size: i32,
    pub snap_start: Option<SnapStart>,
    pub package_type: PackageType,
    pub environment: Environment,
    pub architecture: Architecture,
    pub tags: HashMap<String, String>,
    pub layers: Option<Vec<String>>,
    pub vpc_config: Option<VpcConfig>,
    pub filesystem_config: Option<Vec<FileSystemConfig>>,
    pub _logging_config: Option<LoggingConfig>,
}

impl Function {
    async fn find_arn(&self, client: &Client) -> Option<String> {
        let r = client
            .get_function_configuration()
            .function_name(&self.name)
            .send()
            .await;
        match r {
            Ok(res) => res.function_arn,
            Err(_e) => None,
        }
    }

    async fn get_state(&self, client: &Client, name: &str) -> State {
        let r = client
            .get_function_configuration()
            .function_name(name)
            .send()
            .await;
        match r {
            Ok(res) => res.state.unwrap(),
            Err(_) => State::Failed,
        }
    }

    async fn get_update_status(&self, client: &Client, name: &str) -> LastUpdateStatus {
        let r = client
            .get_function_configuration()
            .function_name(name)
            .send()
            .await;
        match r {
            Ok(res) => res.last_update_status.unwrap(),
            Err(_) => LastUpdateStatus::InProgress,
        }
    }

    async fn wait(&self, client: &Client, name: &str) {
        let mut state: LastUpdateStatus = LastUpdateStatus::InProgress;
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        while state != LastUpdateStatus::Successful {
            state = self.get_update_status(client, name).await;
            let _ = log_update.render(&format!("{} state {}", name, pp_status(&state).blue()));
            sleep(1000)
        }
        let _ = log_update.render(&format!("{} state {}", name, pp_status(&state).green()));
    }

    pub async fn create(&self, client: &Client) -> Result<String> {
        let mut log_update = LogUpdate::new(stdout()).unwrap();

        let name = if kit::trace() {
            &self.name
        } else {
            &self.actual_name
        };

        let _ = log_update.render(&format!(
            "Creating function {} ({})",
            name,
            &self.code_size.cyan()
        ));
        let mut state: State = State::Inactive;

        let f = self.clone();
        let res = client
            .create_function()
            .function_name(&self.name)
            .set_description(self.description.clone())
            .set_runtime(self.runtime.clone())
            .role(&self.role)
            .set_handler(f.handler)
            .code(f.code)
            .environment(f.environment)
            .memory_size(f.memory_size)
            .set_snap_start(f.snap_start)
            .timeout(f.timeout)
            .set_layers(f.layers)
            .package_type(f.package_type)
            .set_tags(Some(f.tags))
            .set_vpc_config(f.vpc_config)
            .architectures(f.architecture)
            .set_file_system_configs(f.filesystem_config)
            .publish(true)
            .send()
            .await?;

        while state != State::Active {
            state = self.get_state(client, &self.name).await;
            let _ = log_update.render(&format!(
                "Checking function {} ({})",
                name,
                pp_state(&state).blue()
            ));
            sleep(800)
        }
        let _ = log_update.render(&format!(
            "Checking function {} ({})",
            name,
            pp_state(&state).green()
        ));

        Ok(res.function_arn.unwrap_or_default())
    }

    pub async fn update_tags(&self, client: &Client, arn: &str) {
        if !&self.tags.is_empty() {
            let res = client
                .tag_resource()
                .resource(arn)
                .set_tags(Some(self.tags.clone()))
                .send()
                .await;
            match res {
                Ok(_) => (),
                Err(_) => println!("error updating tags"),
            }
        }
    }

    pub async fn update_function(&self, client: &Client, arn: &str) -> Result<String, Error> {
        let name = if kit::trace() {
            &self.name
        } else {
            &self.actual_name
        };

        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let _ = log_update.render(&format!(
            "Updating function {} ({})",
            name,
            &self.code_size.cyan()
        ));
        let mut state: LastUpdateStatus = LastUpdateStatus::InProgress;
        while state != LastUpdateStatus::Successful {
            state = self.get_update_status(client, &self.name).await;
            sleep(800)
        }

        let f = self.clone();
        let res = client
            .update_function_configuration()
            .function_name(arn)
            .set_layers(f.layers)
            .role(f.role)
            .set_runtime(f.runtime)
            .set_handler(f.handler)
            .environment(f.environment)
            .timeout(f.timeout)
            .memory_size(f.memory_size)
            .set_snap_start(f.snap_start)
            .set_vpc_config(f.vpc_config)
            .set_file_system_configs(f.filesystem_config)
            .send()
            .await;

        while state != LastUpdateStatus::Successful {
            state = self.get_update_status(client, &self.name).await;
            sleep(800)
        }
        let id = match res {
            Ok(r) => Ok(r.function_arn.unwrap_or_default()),
            Err(e) => panic!("{:?}", e),
        };
        self.update_tags(client, arn).await;

        id
    }

    pub async fn update_code(&self, client: &Client, arn: &str) -> Result<String> {
        let name = if kit::trace() {
            &self.name
        } else {
            &self.actual_name
        };

        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let _ = log_update.render(&format!(
            "Updating function {} ({})",
            name,
            &self.code_size.cyan()
        ));
        let mut state: LastUpdateStatus = LastUpdateStatus::InProgress;

        let res = match self.package_type {
            PackageType::Image => {
                client
                    .update_function_code()
                    .function_name(arn)
                    .image_uri(self.uri.clone())
                    .send()
                    .await?
            }
            PackageType::Zip => {
                client
                    .update_function_code()
                    .function_name(arn)
                    .zip_file(self.blob.clone())
                    .send()
                    .await?
            }
            _ => panic!("unsupported package type"),
        };

        while state != LastUpdateStatus::Successful {
            state = self.get_update_status(client, &self.name).await;
            let _ = log_update.render(&format!(
                "Checking function {} ({})",
                name,
                pp_status(&state).blue()
            ));
            sleep(500)
        }
        let _ = log_update.render(&format!(
            "Checking function {} ({})",
            name,
            pp_status(&state).green()
        ));
        self.update_tags(client, arn).await;
        Ok(res.function_arn.unwrap_or_default())
    }

    pub async fn update_layers(&self, client: &Client, arn: &str) -> Result<String> {
        println!("Updating layer {} {:?}", &self.name, &self.layers);
        let r = client
            .update_function_configuration()
            .function_name(arn)
            .set_layers(self.layers.clone())
            .send()
            .await
            .unwrap();
        self.wait(client, &self.name).await;
        Ok(r.function_arn.unwrap_or_default())
    }

    pub async fn update_vars(&self, client: &Client) -> Result<String> {
        println!("Updating vars {}", &self.name.blue());
        let f = self.clone();
        let r = client
            .update_function_configuration()
            .function_name(&self.name)
            .memory_size(f.memory_size)
            .timeout(f.timeout)
            .environment(f.environment)
            .set_handler(f.handler)
            .send()
            .await?;
        self.wait(client, &self.name).await;
        Ok(r.function_arn.unwrap_or_default())
    }

    pub async fn update_image_vars(&self, client: &Client) -> String {
        println!("Updating vars {}", &self.name);
        let f = self.clone();
        let r = client
            .update_function_configuration()
            .function_name(&self.name)
            .memory_size(f.memory_size)
            .timeout(f.timeout)
            .environment(f.environment)
            .send()
            .await
            .unwrap();
        r.function_arn.unwrap_or_default()
    }

    pub async fn create_or_update(&self, client: &Client) -> String {
        let res = self.find_arn(client).await;
        let arn = match res {
            Some(arn) => {
                let r = self.update_code(client, &arn).await;
                match r {
                    Ok(_) => (),
                    Err(e) => {
                        println!("Failed to update {}", &arn);
                        panic!("{:?}", e);
                    }
                }
                self.update_function(client, &arn).await.unwrap()
            }
            None => self.create(client).await.unwrap(),
        };
        arn
    }

    pub async fn delete(&self, client: &Client) -> Result<()> {
        let mut log_update = LogUpdate::new(stdout()).unwrap();
        let name = if kit::trace() {
            &self.name
        } else {
            &self.actual_name
        };
        let _ = log_update.render(&format!("Deleting function {}", name));
        let mut state: State = State::Active;

        let res = self.find_arn(client).await;

        match res {
            Some(_) => {
                let _ = client
                    .delete_function()
                    .function_name(&self.name)
                    .send()
                    .await?;

                while state == State::Active || state != State::Failed {
                    state = self.get_state(client, &self.name).await;

                    if state != State::Failed {
                        let _ = log_update.render(&format!(
                            "Checking function {} ({})",
                            name,
                            pp_state(&state).blue()
                        ));
                    }
                    sleep(500)
                }
                if state == State::Failed {
                    let _ = log_update.render(&format!(
                        "Checking function {} ({})",
                        name,
                        "ok".green()
                    ));
                }
                Ok(())
            }
            None => {
                let _ = log_update.render(&format!(
                    "Checking function {} ({})",
                    name,
                    "does not exist".red()
                ));
                Ok(())
            }
        }
    }

    pub async fn update_provisioned_concurrency(&self, client: &Client, n: i32) {
        println!("Setting provisioned concurrency {} {}", &self.name, n);
        let _ = client
            .put_provisioned_concurrency_config()
            .function_name(&self.name)
            .qualifier(s!("1"))
            .provisioned_concurrent_executions(n)
            .send()
            .await
            .unwrap();
    }

    pub async fn update_reserved_concurrency(&self, client: &Client, n: i32) {
        println!("Setting reserved concurrency {} {}", &self.name, n);
        let _ = client
            .put_function_concurrency()
            .function_name(&self.name)
            .reserved_concurrent_executions(n)
            .send()
            .await
            .unwrap();
    }

    async fn find_alias(&self, client: &Client) -> Option<String> {
        let res = client
            .get_alias()
            .name(self.name.to_string())
            .function_name(&self.name)
            .send()
            .await;
        match res {
            Ok(r) => r.name,
            Err(_) => None,
        }
    }

    async fn update_alias(&self, client: &Client, version: &str) {
        let _ = client
            .update_alias()
            .name(self.name.to_string())
            .function_name(s!(self.name))
            .function_version(version)
            .send()
            .await;
    }

    async fn create_alias(&self, client: &Client, version: &str) {
        let _ = client
            .create_alias()
            .name(self.name.to_string())
            .function_name(&self.name)
            .function_version(version)
            .send()
            .await;
    }

    pub async fn publish_version(&self, client: &Client) {
        self.wait(client, &self.name).await;
        let res = client
            .publish_version()
            .function_name(&self.name)
            .send()
            .await;
        let version = res.unwrap().version.unwrap();

        let maybe_alias = self.find_alias(client).await;
        match maybe_alias {
            Some(_) => self.update_alias(client, &version).await,
            None => self.create_alias(client, &version).await,
        }

        println!("Published alias {} with version ({})", &self.name, &version);
    }
}

pub async fn add_permission(
    client: Client,
    name: &str,
    principal: &str,
    source_arn: &str,
    statement_id: &str,
) -> Result<()> {
    let _res = client
        .add_permission()
        .function_name(name.to_string())
        .statement_id(s!(statement_id))
        .action(s!("lambda:InvokeFunction"))
        .principal(principal.to_string())
        .source_arn(source_arn.to_string())
        .send()
        .await?;
    //println!("{:?}", res);

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

pub async fn update_tags(client: &Client, arn: &str, tags: &HashMap<String, String>) {
    println!("Updating tags {} {:?}", arn, &tags);
    let res = client
        .tag_resource()
        .resource(arn)
        .set_tags(Some(tags.clone()))
        .send()
        .await;
    match res {
        Ok(_) => (),
        Err(_) => println!("error updating tags"),
    }
}

pub fn _make_deadletter(sqs_arn: &str) -> DeadLetterConfig {
    let v = DeadLetterConfigBuilder::default();
    v.set_target_arn(Some(s!(sqs_arn))).build()
}

pub async fn _update_dlq(client: &Client, name: &str, sqs_arn: &str) {
    let config = _make_deadletter(sqs_arn);
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

pub async fn update_runtime_management_config(client: &Client, name: &str, version: &str) {
    println!("Updating runtime ({}): Manual", &name);
    let res = client
        .put_runtime_management_config()
        .function_name(s!(name))
        .update_runtime_on(UpdateRuntimeOn::Manual)
        .runtime_version_arn(s!(version))
        .send()
        .await;
    match res {
        Ok(_) => (),
        Err(_) => panic!("{:?}", res),
    }
}

pub async fn list_tags(client: &Client, arn: &str) -> Result<HashMap<String, String>, Error> {
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

pub async fn get_tag(client: &Client, arn: &str, tag: String) -> String {
    let tags = list_tags(&client, arn).await.unwrap();
    match tags.get(&tag) {
        Some(v) => v.to_string(),
        None => "".to_string(),
    }
}

pub struct Config {
    pub code_size: i64,
    pub timeout: i32,
    pub mem_size: i32,
    pub role: String,
    pub package_type: String,
}

/// Update only the IAM role on an existing lambda. Idempotent: if the
/// configured role already matches `role_arn`, the call is skipped so
/// we don't burn a needless `update_function_configuration` API call.
/// Returns `Ok(false)` if the function doesn't exist (caller should
/// rely on the create path to set the role) or already had the right
/// role; `Ok(true)` if we issued an update.
pub async fn update_role(client: &Client, name: &str, role_arn: &str) -> Result<bool, Error> {
    let current = find_config(client, name).await;
    let needs_update = match current {
        Some(cfg) => cfg.role != role_arn,
        None => return Ok(false),
    };
    if !needs_update {
        return Ok(false);
    }
    // Wait for any in-flight update to complete before issuing ours —
    // AWS returns ResourceConflictException otherwise.
    let mut state = LastUpdateStatus::InProgress;
    let mut tries = 0;
    while state == LastUpdateStatus::InProgress && tries < 60 {
        let r = client
            .get_function_configuration()
            .function_name(s!(name))
            .send()
            .await;
        match r {
            Ok(res) => {
                state = res
                    .last_update_status
                    .unwrap_or(LastUpdateStatus::Successful)
            }
            Err(_) => break,
        }
        if state == LastUpdateStatus::InProgress {
            sleep(800);
        }
        tries += 1;
    }
    let res = client
        .update_function_configuration()
        .function_name(s!(name))
        .role(s!(role_arn))
        .send()
        .await;
    match res {
        Ok(_) => Ok(true),
        Err(e) => Err(e.into()),
    }
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
                role: res.role.unwrap(),
                package_type: res.package_type.unwrap().to_string(),
            };
            Some(cfg)
        }
        Err(_e) => None,
    }
}

pub async fn find_uri(client: &Client, name: &str) -> Option<String> {
    let r = client.get_function().function_name(s!(name)).send().await;
    match r {
        Ok(res) => res.code.unwrap().image_uri,
        Err(_) => None,
    }
}

pub async fn delete_by_arn(client: &Client, arn: &str) {
    println!("Deleting {}", arn);
    let _ = client
        .delete_function()
        .function_name(arn)
        .send()
        .await
        .unwrap();
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

pub async fn invoke(client: Client, name: &str, payload: &str) -> Result<()> {
    let blob = make_blob_from_str(payload);
    let r = client
        .invoke()
        .function_name(name)
        .payload(blob)
        .invocation_type(InvocationType::RequestResponse)
        .log_type(LogType::Tail)
        .send()
        .await?;

    print_logs(r.log_result, r.payload);
    Ok(())
}

pub async fn invoke_sync(client: &Client, name: &str, payload: &str) -> Result<String> {
    let blob = make_blob_from_str(payload);
    let r = client
        .invoke()
        .function_name(name)
        .payload(blob)
        .invocation_type(InvocationType::RequestResponse)
        .log_type(LogType::Tail)
        .send()
        .await?;

    match r.payload {
        Some(p) => Ok(String::from_utf8_lossy(&p.into_inner()).to_string()),
        _ => Ok(String::from("")),
    }
}

pub async fn invoke_async(client: &Client, name: &str, payload: &str) -> Result<String> {
    let blob = make_blob_from_str(payload);
    let r = client
        .invoke()
        .function_name(name)
        .payload(blob)
        .invocation_type(InvocationType::Event)
        .send()
        .await?;

    match r.payload {
        Some(p) => Ok(String::from_utf8_lossy(&p.into_inner()).to_string()),
        _ => Ok(String::from("")),
    }
}

fn make_destination_config(arn: &str) -> DestinationConfig {
    let os = OnSuccessBuilder::default();
    let on_success = os.destination(arn).build();
    let f = DestinationConfigBuilder::default();
    f.on_success(on_success).build()
}

pub async fn update_destination(client: &Client, name: &str, target_arn: &str) {
    let dest_config = make_destination_config(target_arn);
    let _ = client
        .put_function_event_invoke_config()
        .function_name(name)
        .destination_config(dest_config)
        .send()
        .await
        .unwrap();
}

pub async fn find_alias_arn(client: &Client, name: &str) -> Option<String> {
    let res = client
        .get_alias()
        .name(name)
        .function_name(name)
        .send()
        .await;
    match res {
        Ok(r) => r.alias_arn,
        Err(_) => None,
    }
}

pub type LambdaClient = Client;
