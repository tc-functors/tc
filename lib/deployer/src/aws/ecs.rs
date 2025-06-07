use authorizer::Auth;
use aws_sdk_ecs::{
    Client,
    types::{
        CapacityProviderStrategyItem,
        Compatibility,
        ContainerDefinition,
        EfsVolumeConfiguration,
        LaunchType,
        NetworkConfiguration,
        NetworkMode,
        SchedulingStrategy,
        Volume,
        builders::{
            AwsVpcConfigurationBuilder,
            CapacityProviderStrategyItemBuilder,
            ContainerDefinitionBuilder,
            NetworkConfigurationBuilder,
            VolumeBuilder,
        },
    },
};
use kit::*;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub struct TaskDef {
    pub name: String,
    pub task_role_arn: String,
    pub network_mode: NetworkMode,
    pub cpu: String,
    pub mem: String,
}

impl TaskDef {
    pub fn new(name: &str, arn: &str, mem: &str, cpu: &str) -> TaskDef {
        TaskDef {
            name: s!(name),
            task_role_arn: s!(arn),
            cpu: s!(cpu),
            mem: s!(mem),
            network_mode: NetworkMode::Awsvpc,
        }
    }
}

pub fn make_cdf(name: &str, image: &str, command: &str) -> ContainerDefinition {
    let f = ContainerDefinitionBuilder::default();
    f
        .name(s!(name))
        .image(s!(image))
        .command(s!(command))
        .build()
}

pub fn make_network_config(subnets: Vec<String>) -> NetworkConfiguration {
    let v = AwsVpcConfigurationBuilder::default();
    let vpc = v.set_subnets(Some(subnets)).build().unwrap();
    let net = NetworkConfigurationBuilder::default();
    net.awsvpc_configuration(vpc).build()
}

pub fn make_volume(name: String, efs_config: EfsVolumeConfiguration) -> Volume {
    let f = VolumeBuilder::default();
    f.name(name).efs_volume_configuration(efs_config).build()
}

pub fn make_capacity_provider() -> CapacityProviderStrategyItem {
    let f = CapacityProviderStrategyItemBuilder::default();
    f.capacity_provider("s!(FARGATE)")
        .weight(1)
        .base(0)
        .build()
        .unwrap()
}

pub async fn create_taskdef(client: &Client, tdf: TaskDef, cdf: ContainerDefinition) -> String {
    let res = client
        .register_task_definition()
        .family(tdf.name)
        .task_role_arn(tdf.task_role_arn.to_owned())
        .execution_role_arn(tdf.task_role_arn)
        .network_mode(tdf.network_mode)
        .cpu(tdf.cpu)
        .memory(tdf.mem)
        .requires_compatibilities(Compatibility::Fargate)
        .container_definitions(cdf)
        .send()
        .await;
    match res {
        Ok(r) => match r.task_definition {
            Some(t) => t.task_definition_arn.unwrap(),
            None => panic!("failed to create task def"),
        },
        Err(_) => {
            println!("{:?}", res);
            panic!("failed to create task def")
        }
    }
}

async fn list_clusters(client: &Client) -> Vec<String> {
   let res = client
        .list_clusters()
        .send()
        .await
        .unwrap();
    res.cluster_arns.unwrap()
}

async fn find_cluster(client: &Client, name: &str) -> Option<String> {
    let cluster_arns = list_clusters(client).await;
    for arn in cluster_arns {
        if arn.ends_with(name) {
            return Some(name.to_string())
        }
    }
    None
}

async fn create_cluster(client: &Client, name: &str) -> String {
    let res = client
        .create_cluster()
        .cluster_name(s!(name))
        .send()
        .await;
    println!("{:?}", res);
    s!(name)
}


pub async fn find_or_create_cluster(client: &Client, name: &str) -> String {
    let cluster = find_cluster(client, name).await;
    match cluster {
        Some(c) => c,
        None => create_cluster(client, name).await
    }
}


pub async fn create_service(
    client: &Client,
    cluster: &str,
    name: &str,
    task_definition_arn: &str,
    netcfg: NetworkConfiguration,
) {
    let _ = client
        .create_service()
        .cluster(s!(cluster))
        .service_name(s!(name))
        .task_definition(s!(task_definition_arn))
        .launch_type(LaunchType::Fargate)
        .desired_count(1)
        .network_configuration(netcfg)
        .scheduling_strategy(SchedulingStrategy::Replica)
        .send()
        .await;
}

pub async fn delete_service(client: &Client, cluster: &str, name: &str) {
    let _ = client
        .delete_service()
        .cluster(s!(cluster))
        .service(s!(name))
        .force(true)
        .send()
        .await;
}


pub async fn run_task(
    client: &Client,
    cluster: &str,
    name: &str,
    task_definition_arn: &str,
    netcfg: NetworkConfiguration,
) {
    let _res = client
        .run_task()
        .cluster(cluster)
        .task_definition(s!(task_definition_arn))
        .launch_type(LaunchType::Fargate)
        .count(1)
        .network_configuration(netcfg)
        .send()
        .await;
}
