use crate::Auth;
use aws_sdk_resourcegroupstagging::{
    Client,
    types::{
        TagFilter,
        builders::TagFilterBuilder,
    },
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

fn make_filters(key: &str, val: &str) -> TagFilter {
    let b = TagFilterBuilder::default();
    b.key(key.to_string()).values(val.to_string()).build()
}

async fn get_resources_by_token(
    client: &Client,
    token: &str,
    filters: TagFilter,
) -> (Vec<String>, Option<String>) {
    let res = client
        .get_resources()
        .pagination_token(token.to_string())
        .resources_per_page(100)
        .tag_filters(filters)
        .send()
        .await
        .unwrap();

    let mut arns: Vec<String> = vec![];
    let mappings = res.resource_tag_mapping_list.unwrap().to_vec();
    for m in mappings {
        arns.push(m.resource_arn.unwrap());
    }
    (arns, res.pagination_token)
}

pub async fn get_resources(client: &Client, key: &str, val: &str) -> Vec<String> {
    let filters = make_filters(key, val);
    let res = client
        .get_resources()
        .resources_per_page(100)
        .tag_filters(filters.clone())
        .send()
        .await
        .unwrap();

    let mut arns: Vec<String> = vec![];
    let mut token: Option<String> = res.pagination_token;

    let mappings = res.resource_tag_mapping_list.unwrap().to_vec();
    for m in mappings {
        arns.push(m.resource_arn.unwrap());
    }

    match token {
        Some(tk) => {
            token = Some(tk);
            while token.is_some() {
                let (xs, t) =
                    get_resources_by_token(client, &token.unwrap(), filters.clone()).await;
                arns.extend(xs.clone());
                token = t.clone();
                if let Some(x) = t {
                    if x.is_empty() {
                        break;
                    }
                }
            }
        }
        None => (),
    }
    arns
}
