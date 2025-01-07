use anyhow::{Error, Result};
use aws_sdk_scheduler::types::builders::FlexibleTimeWindowBuilder;
use aws_sdk_scheduler::types::builders::TargetBuilder;
use aws_sdk_scheduler::types::{FlexibleTimeWindow, FlexibleTimeWindowMode, ScheduleState, Target};
use aws_sdk_scheduler::Client;

use super::Env;
use colored::Colorize;
use kit::*;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

fn make_time_window() -> FlexibleTimeWindow {
    let m = FlexibleTimeWindowBuilder::default();
    m.mode(FlexibleTimeWindowMode::Flexible)
        .maximum_window_in_minutes(10)
        .build()
        .unwrap()
}

pub fn make_target(arn: &str, role_arn: &str, kind: &str, input: &str) -> Target {
    let target = TargetBuilder::default();

    match kind {
        "sfn" => target
            .arn(arn)
            .role_arn(role_arn)
            .input(s!(input))
            .build()
            .unwrap(),
        "lambda" => target.arn(arn).input(s!(input)).build().unwrap(),
        _ => target
            .arn(arn)
            .role_arn(role_arn)
            .input(s!(input))
            .build()
            .unwrap(),
    }
}

async fn create_schedule_group(client: &Client, name: &str) -> Result<()> {
    println!("Creating schedule group {}", name.green());
    let _ = client
        .create_schedule_group()
        .name(s!(name))
        .send()
        .await
        .unwrap();
    Ok(())
}

async fn get_schedule_group(client: &Client, name: &str) -> Result<Option<String>> {
    let r = client.get_schedule_group().name(s!(name)).send().await?;
    Ok(r.arn)
}

pub async fn find_or_create_group(client: &Client, name: &str) {
    let group = get_schedule_group(client, name).await;
    match group {
        Ok(_) => (),
        Err(_) => {
            let _ = create_schedule_group(client, name).await;
        }
    }
}

pub async fn _delete_schedule_group(client: &Client, group: &str) -> Result<()> {
    let _ = client
        .delete_schedule_group()
        .name(s!(group))
        .send()
        .await?;
    Ok(())
}

pub async fn delete_schedule(client: &Client, group: &str, name: &str) -> Result<(), Error> {
    println!("Deleting schedule {}/{}", group, name.red());
    let _ = client
        .delete_schedule()
        .name(name)
        .group_name(group)
        .send()
        .await;
    Ok(())
}

async fn create_schedule(
    client: &Client,
    group: &str,
    name: &str,
    target: Target,
    expression: &str,
) -> Result<(), Error> {
    println!(
        "Creating schedule {}/{} ({})",
        group,
        name.green(),
        expression
    );
    let _ = client
        .create_schedule()
        .name(name)
        .group_name(group)
        .schedule_expression(expression)
        .state(ScheduleState::Enabled)
        .flexible_time_window(make_time_window())
        .target(target)
        .send()
        .await
        .unwrap();
    Ok(())
}

async fn update_schedule(
    client: &Client,
    group: &str,
    name: &str,
    target: Target,
    expression: &str,
) -> Result<()> {
    println!(
        "Updating schedule {}/{} ({})",
        group,
        name.blue(),
        expression
    );
    let _ = client
        .update_schedule()
        .name(name)
        .group_name(group)
        .schedule_expression(expression)
        .flexible_time_window(make_time_window())
        .state(ScheduleState::Enabled)
        .target(target)
        .send()
        .await
        .unwrap();
    Ok(())
}

pub async fn get_schedule(client: &Client, name: &str, group: &str) -> Result<Option<String>> {
    let r = client
        .get_schedule()
        .name(name)
        .group_name(group)
        .send()
        .await;
    match r {
        Ok(res) => Ok(res.arn),
        Err(_) => Ok(None),
    }
}

pub async fn create_or_update_schedule(
    client: &Client,
    group: &str,
    name: &str,
    target: Target,
    expression: &str,
) -> Result<()> {
    let s = get_schedule(client, name, group).await.unwrap();
    match s {
        Some(_) => update_schedule(client, group, name, target, expression).await,
        None => create_schedule(client, group, name, target, expression).await,
    }
}
