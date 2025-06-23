use crate::aws;
use authorizer::Auth;
use compiler::Schedule;
use std::collections::HashMap;

pub async fn create_schedule(auth: &Auth, schedule: &Schedule) {
    let client = aws::scheduler::make_client(&auth).await;
    let Schedule {
        name,
        group,
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
            group,
            &name,
            target,
            &expression,
        )
        .await;
    }
}

pub async fn create(auth: &Auth, schedules: &HashMap<String, Schedule>) {
    let client = aws::scheduler::make_client(&auth).await;
    for (_, schedule) in schedules {
        aws::scheduler::find_or_create_group(&client, &schedule.group).await;
        create_schedule(&auth, schedule).await;
    }
}

pub async fn delete(auth: &Auth, schedules: &HashMap<String, Schedule>) {
    let client = aws::scheduler::make_client(&auth).await;
    for (_, schedule) in schedules {
        let _ = aws::scheduler::delete_schedule(&client, &schedule.group, &schedule.name).await;
    }
}

pub async fn update(_auth: &Auth, _schedules: &HashMap<String, Schedule>) {

}
