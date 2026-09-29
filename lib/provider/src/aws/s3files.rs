use super::constants;
use crate::Auth;
pub use aws_sdk_s3files::Client;
use aws_sdk_s3files::{
    config,
    config::retry::{
        RetryConfig,
        RetryMode,
    },
    types::LifeCycleState,
    types::builders::{
        PosixUserBuilder,
        CreationPermissionsBuilder,
        RootDirectoryBuilder
    }
};
use kit as u;

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
                    .with_max_backoff(constants::MAX_BACKOFF),
            )
            .build(),
    )
}

// fs
async fn find_fs(client: &Client, bucket: &str) -> Option<String> {
    let res = client
        .list_file_systems()
        .bucket(bucket)
        .send()
        .await
        .unwrap();
    let maybe_fs = res.file_systems.first();
    if let Some(fs) = maybe_fs.as_ref() {
        Some(fs.file_system_id.clone())
    } else {
        None
    }
}

async fn get_fs_state(client: &Client, fs_id: &str) -> LifeCycleState {
    let res = client
        .get_file_system()
        .file_system_id(fs_id)
        .send()
        .await
        .unwrap();
    res.status.unwrap()
}

async fn create_fs(client: &Client, bucket: &str, role_arn: &str) -> String {
    let res = client
        .create_file_system()
        .bucket(bucket)
        .role_arn(role_arn)
        .accept_bucket_warning(true)
        .send()
        .await
        .unwrap();

    let fs_id = res.file_system_id.unwrap();
    let mut state: LifeCycleState = get_fs_state(client, &fs_id).await;
    while state != LifeCycleState::Available {
        u::sleep(5000);
        state = get_fs_state(client, &fs_id).await;
    }

    fs_id
}

pub async fn find_or_create_fs(client: &Client, bucket: &str, role_arn: &str) -> String {
    let maybe_fs_id = find_fs(client, bucket).await;
    match maybe_fs_id {
        Some(id) => id,
        None => create_fs(client, bucket, role_arn).await
    }
}

// ap
async fn find_ap(client: &Client, fs_id: &str) -> Option<String> {
    let res = client
        .list_access_points()
        .file_system_id(fs_id)
        .send()
        .await
        .unwrap();

    let xs = res.access_points.to_vec();
    for x in xs {
        if x.file_system_id == fs_id {
            return Some(x.access_point_arn)
        }
    }
    None
}

async fn get_ap_state(client: &Client, id: &str) -> LifeCycleState {
    let res = client
        .get_access_point()
        .access_point_id(id)
        .send()
        .await
        .unwrap();
    res.status
}

async fn create_ap(client: &Client, fs_id: &str) -> String {
    let pu = PosixUserBuilder::default();
    let posix_user = pu.uid(1000).gid(1000).build().unwrap();

    let cp = CreationPermissionsBuilder::default();
    let perm = cp.owner_uid(1000).owner_gid(1000).permissions("755").build().unwrap();

    let rd = RootDirectoryBuilder::default();
    let root_dir = rd.path("/lambda").creation_permissions(perm).build();

    let res = client
        .create_access_point()
        .file_system_id(fs_id)
        .posix_user(posix_user)
        .root_directory(root_dir)
        .send()
        .await
        .unwrap();

    let ap_id = res.access_point_id;
    let mut state: LifeCycleState = get_ap_state(client, &ap_id).await;
    while state != LifeCycleState::Available {
        u::sleep(5000);
        state = get_ap_state(client, &ap_id).await;
    }

    res.access_point_arn
}

pub async fn find_or_create_ap(client: &Client, fs_id: &str) -> String {
    let maybe_ap_arn = find_ap(client, fs_id).await;
    match maybe_ap_arn {
        Some(ap_arn) => ap_arn,
        None => create_ap(client, fs_id).await
    }
}

// mount_target

async fn find_mt(client: &Client, fs_id: &str, subnet_id: &str) -> Option<String> {
    let res = client
        .list_mount_targets()
        .file_system_id(fs_id)
        .send()
        .await
        .unwrap();

    let xs = res.mount_targets.to_vec();
    for x in xs {
        if x.file_system_id.unwrap() == fs_id && x.subnet_id == subnet_id {
            return Some(x.mount_target_id)
        }
    }
    None
}

async fn create_mt(client: &Client, fs_id: &str, subnet_id: &str, sgs: Vec<String>) -> String {
    let res = client
        .create_mount_target()
        .file_system_id(fs_id)
        .subnet_id(subnet_id)
        .set_security_groups(Some(sgs))
        .send()
        .await
        .unwrap();
    res.mount_target_id
}

pub async fn find_or_create_mt(client: &Client, fs_id: &str, subnet_id: &str, sgs: Vec<String>) -> String {
    let maybe_mt = find_mt(client, fs_id, subnet_id).await;
    match maybe_mt {
        Some(id) => id,
        None => create_mt(client, fs_id, subnet_id, sgs).await
    }
}
