use crate::resolver::Schedule;
use aws::scheduler;
use aws::Env;

pub async fn create_schedule(env: &Env, namespace: &str, schedule: Schedule) {
    let client = scheduler::make_client(&env).await;
    let Schedule {
        name,
        target_arn,
        role_arn,
        expression,
        payload,
        ..
    } = schedule;

    if !target_arn.is_empty() {
        let target = scheduler::make_target(&target_arn, &role_arn, "sfn", &payload);
        let _ =
            scheduler::create_or_update_schedule(&client, namespace, &name, target, &expression)
                .await;
    }
}

pub async fn create(env: &Env, namespace: &str, schedules: Vec<Schedule>) {
    let client = scheduler::make_client(&env).await;
    scheduler::find_or_create_group(&client, namespace).await;
    for schedule in schedules {
        create_schedule(&env, namespace, schedule).await;
    }
}

pub async fn delete(env: &Env, namespace: &str, schedules: Vec<Schedule>) {
    let client = scheduler::make_client(&env).await;
    for schedule in schedules {
        let _ = scheduler::delete_schedule(&client, namespace, &schedule.name).await;
    }
}
