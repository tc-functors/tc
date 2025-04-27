
pub async fn find_or_create(client: &Client, api_id: &str, sqs_arn: &str) -> String {
    let maybe_int = find(api_id).await;
    match maybe_int {
        Some(id) => id,
        _ => create(api_id, sfn_arn).await.unwrap(),
    }
}
