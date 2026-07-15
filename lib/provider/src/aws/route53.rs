use crate::Auth;
use aws_sdk_route53::{
    Client,
    config,
    config::retry::{RetryConfig, RetryMode},
    types::{
        AliasTarget,
        Change,
        ChangeAction,
        ChangeBatch,
        ResourceRecord,
        ResourceRecordSet,
        RrType,
        builders::{
            AliasTargetBuilder,
            ChangeBatchBuilder,
            ChangeBuilder,
            ResourceRecordBuilder,
            ResourceRecordSetBuilder,
        },
    },
};
use super::constants;
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.get_global_config().await;
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

pub struct ValidationRecord {
    pub name: String,
    pub rtype: RrType,
    pub value: String,
}

fn make_resource_record(value: &str) -> ResourceRecord {
    let b = ResourceRecordBuilder::default();
    b.value(value).build().unwrap()
}

fn make_record_set(vr: ValidationRecord) -> ResourceRecordSet {
    let ValidationRecord { rtype, value, name } = vr;
    let b = ResourceRecordSetBuilder::default();
    let resource_record = make_resource_record(&value);
    b.name(name)
        .r#type(rtype)
        .resource_records(resource_record)
        .ttl(300)
        .build()
        .unwrap()
}

fn make_alias_target(target_zone_id: &str, dns_name: &str) -> AliasTarget {
    let b = AliasTargetBuilder::default();
    b.hosted_zone_id(target_zone_id)
        .dns_name(dns_name)
        .build()
        .unwrap()
}

fn make_alias_record_set(vr: ValidationRecord, target_zone_id: &str) -> ResourceRecordSet {
    let ValidationRecord { value, name, .. } = vr;
    let b = ResourceRecordSetBuilder::default();
    let alias_target = make_alias_target(target_zone_id, &value);
    b.name(name)
        .r#type(RrType::from("A"))
        .alias_target(alias_target)
        .build()
        .unwrap()
}

fn make_change(vr: ValidationRecord, target_zone_id: &str, root: bool) -> Change {
    let b = ChangeBuilder::default();
    let record_set = if root {
        make_alias_record_set(vr, target_zone_id)
    } else {
        make_record_set(vr)
    };
    b.action(ChangeAction::Upsert)
        .resource_record_set(record_set)
        .build()
        .unwrap()
}

fn make_change_batch(
    vr: ValidationRecord,
    target_zone_id: Option<String>,
    root: bool,
) -> ChangeBatch {
    let b = ChangeBatchBuilder::default();
    let tz_id = if root {
        match target_zone_id {
            Some(t) => t,
            None => panic!("No target_zone_id specified"),
        }
    } else {
        // non-root does not need tz_id
        String::from("")
    };
    let change = make_change(vr, &tz_id, root);
    b.set_comment(None).changes(change).build().unwrap()
}

async fn list_hosted_zones(client: &Client) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let res = client.list_hosted_zones().send().await.unwrap();
    let zones = res.hosted_zones;
    for zone in zones {
        h.insert(zone.name, zone.id);
    }
    h
}

async fn get_hosted_zone_id(client: &Client, name: &str) -> (Option<String>, bool) {
    let zname = if let Some((_k, v)) = name.split_once(".") {
        format!("{}.", &v)
    } else {
        String::from("")
    };

    let fname = format!("{}.", name);

    let zones = list_hosted_zones(client).await;

    tracing::debug!("{:?} {} {}", &zones, zname, fname);

    if let Some(zone_id) = zones.get(&fname) {
        let parts: Vec<&str> = zone_id.split("/").collect();
        (parts.clone().last().cloned().map(String::from), true)
    } else if let Some(zone_id) = zones.get(&zname) {
        let parts: Vec<&str> = zone_id.split("/").collect();
        (
            parts.clone().last().cloned().map(String::from),
            &zname == &fname,
        )
    } else {
        (None, false)
    }
}

pub async fn create_validation_record_set(
    client: &Client,
    domain: &str,
    name: &str,
    rtype: &str,
    value: &str,
) -> (Option<String>, bool) {
    let (maybe_hosted_zone_id, root) = get_hosted_zone_id(client, domain).await;
    println!(
        "Creating Validation Recordset {} {} {:?} root:{}",
        domain, name, maybe_hosted_zone_id, root
    );

    if let Some(hosted_zone_id) = maybe_hosted_zone_id.clone() {
        let vr = ValidationRecord {
            name: name.to_string(),
            rtype: RrType::from(rtype),
            value: value.to_string(),
        };
        let change_batch = make_change_batch(vr, None, false);
        let _ = client
            .change_resource_record_sets()
            .hosted_zone_id(hosted_zone_id)
            .change_batch(change_batch)
            .send()
            .await
            .unwrap();
    } else {
        panic!("Hosted zone id not found");
    }
    (maybe_hosted_zone_id, root)
}

pub async fn create_record_set(
    client: &Client,
    domain: &str,
    rtype: &str,
    value: &str,
    target_zone_id: Option<String>,
) {
    let (maybe_hosted_zone_id, root) = get_hosted_zone_id(client, domain).await;
    println!("Creating Recordset {} root:{}", domain, root);
    if let Some(hosted_zone_id) = maybe_hosted_zone_id {
        let vr = ValidationRecord {
            name: domain.to_string(),
            rtype: RrType::from(rtype),
            value: value.to_string(),
        };
        let change_batch = make_change_batch(vr, target_zone_id, root);
        let _ = client
            .change_resource_record_sets()
            .hosted_zone_id(hosted_zone_id)
            .change_batch(change_batch)
            .send()
            .await
            .unwrap();
    } else {
        panic!("Hosted zone id not found");
    }
}
