use crate::Auth;
use aws_sdk_route53::Client;
use aws_sdk_route53::types::ChangeBatch;
use aws_sdk_route53::types::builders::ChangeBatchBuilder;
use aws_sdk_route53::types::builders::ChangeBuilder;
use aws_sdk_route53::types::ChangeAction;
use aws_sdk_route53::types::Change;
use aws_sdk_route53::types::ResourceRecordSet;
use aws_sdk_route53::types::RrType;
use aws_sdk_route53::types::builders::ResourceRecordSetBuilder;
use aws_sdk_route53::types::ResourceRecord;
use aws_sdk_route53::types::builders::ResourceRecordBuilder;
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.get_global_config().await;
    Client::new(shared_config)
}

pub struct ValidationRecord {
    pub name: String,
    pub rtype: RrType,
    pub value: String
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

fn make_change(vr: ValidationRecord) -> Change {
    let b = ChangeBuilder::default();
    let record_set = make_record_set(vr);
    b.action(ChangeAction::Upsert).resource_record_set(record_set).build().unwrap()
}

fn make_change_batch(vr: ValidationRecord) -> ChangeBatch {
    let b = ChangeBatchBuilder::default();
    let change = make_change(vr);
    b.set_comment(None).changes(change).build().unwrap()
}

async fn list_hosted_zones(client: &Client) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let res = client.list_hosted_zones().send().await.unwrap();
    let zones = res.hosted_zones;
    for zone in zones {
        h.insert(zone.name,  zone.id);
    }
    h
}

async fn get_hosted_zone_id(client: &Client, name: &str) -> Option<String> {
    let zname = if let Some((_k, v)) = name.split_once(".") {
        format!("{}.", &v)
    } else {
        panic!("Invalid domain. Can't get zone id")
    };

    let zones = list_hosted_zones(client).await;
    let maybe_id = zones.get(&zname);

    if let Some(id) = maybe_id {
        let parts: Vec<&str> = id.split("/").collect();
        parts.clone().last().cloned().map(String::from)
    } else {
        None
    }
}

pub async fn create_record_set(client: &Client, domain: &str, name: &str, rtype: &str, value: &str) {
    tracing::debug!("Creating Recordset {} {} {}", name, rtype, value);
    let maybe_hosted_zone_id = get_hosted_zone_id(client, domain).await;

    if let Some(hosted_zone_id) = maybe_hosted_zone_id {
        let vr = ValidationRecord {
            name: name.to_string(),
            rtype: RrType::from(rtype),
            value: value.to_string()
        };
        let change_batch = make_change_batch(vr);
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
