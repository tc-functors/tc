use crate::aws;
use authorizer::Auth;
use compiler::Schedule;
use std::collections::HashMap;

pub async fn create_schedule(auth: &Auth, namespace: &str, schedule: Schedule) {
    let client = aws::scheduler::make_client(&auth).await;
    let Schedule {
        name,
        target_arn,
        role_arn,
        expression,
        payload,
        ..
    } = schedule;

    if !target_arn.is_empty() {
        let target = aws::scheduler::make_target(&target_arn, &role_arn, "sfn", &payload);
        let _ = aws::scheduler::create_or_update_schedule(
            &client,
            namespace,
            &name,
            target,
            &expression,
        )
        .await;
    }
}

pub async fn create(auth: &Auth, namespace: &str, schedules: HashMap<String, Schedule>) {
    let client = aws::scheduler::make_client(&auth).await;
    aws::scheduler::find_or_create_group(&client, namespace).await;
    for (_, schedule) in schedules {
        create_schedule(&auth, namespace, schedule).await;
    }
}

pub async fn delete(auth: &Auth, namespace: &str, schedules: HashMap<String, Schedule>) {
    let client = aws::scheduler::make_client(&auth).await;
    for (_, schedule) in schedules {
        let _ = aws::scheduler::delete_schedule(&client, namespace, &schedule.name).await;
    }
}
