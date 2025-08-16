use std::collections::{
    HashMap,
    hash_map::Entry,
};
use authorizer::Auth;
use composer::Entity;
use crate::aws::resourcetag;
use crate::aws;

pub async fn list(auth: &Auth, sandbox: &str) -> Vec<String> {
    let client = resourcetag::make_client(auth).await;
    resourcetag::get_resources(&client, "sandbox", sandbox).await

}

pub fn group_entities(arns: Vec<String>) -> HashMap<Entity, Vec<String>> {
    let mut h: HashMap<Entity, Vec<String>> = HashMap::new();
    for arn in arns {
        let maybe_entity = Entity::from_arn(&arn);
        if let Some(entity) = maybe_entity {
            match h.entry(entity) {
                Entry::Vacant(e) => {
                    e.insert(vec![arn]);
                }
                Entry::Occupied(mut e) => {
                    if !e.get_mut().contains(&arn) {
                        e.get_mut().push(arn);
                    }
                }
            }
        }
    }
    h
}

pub fn count_of(grouped: &HashMap<Entity, Vec<String>>) -> String {
    let mut f: usize = 0;
    let mut s: usize = 0;
    let mut m: usize = 0;
    let mut r: usize = 0;
    let mut e: usize = 0;
    for (entity, arns) in grouped {
        match entity {
            Entity::Function => f = arns.len(),
            Entity::State => s = arns.len(),
            Entity::Mutation => m = arns.len(),
            Entity::Route => r = arns.len(),
            Entity::Event => e = arns.len(),
            _ => (),
        }
    }
    format!(
        "Found functions:{}, states:{}, mutations:{}, routes:{}, events:{}",
        f, s, m, r, e
    )
}

pub fn filter_arns(arns: Vec<String>, filter: Option<String>) -> Vec<String> {
    match filter {
        Some(f) => {
            let mut xs: Vec<String> = vec![];
            for arn in arns {
                if arn.contains(&f) {
                    xs.push(arn);
                }
            }
            xs
        },
        None => arns
    }
}


pub async fn delete_arns(auth: &Auth, grouped: HashMap<Entity, Vec<String>>) {
    for (entity, arns) in grouped {
        match entity {
            Entity::Function => {
                let client = aws::lambda::make_client(auth).await;
                for arn in arns {
                    aws::lambda::delete_by_arn(&client, &arn).await;
                }
            }
            Entity::State => {
                let client = aws::sfn::make_client(auth).await;
                for arn in arns {
                    aws::sfn::delete_by_arn(&client, &arn).await;
                }
            }

            Entity::Mutation => {
                let client = aws::appsync::make_client(auth).await;
                for arn in arns {
                    let api_id = kit::split_last(&arn, "/");
                    aws::appsync::delete_by_id(&client, &api_id).await;
                }
            }
            _ => println!("Skipping entity {}", &entity.to_str()),
        }
    }
}
