mod aws;

use authorizer::Auth;
use compiler::Entity;
use question::{
    Answer,
    Question,
};
use std::collections::{
    HashMap,
    hash_map::Entry,
};

fn group_entities(arns: Vec<String>) -> HashMap<Entity, Vec<String>> {
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

fn maybe_continue() -> bool {
    let answer = Question::new("Do you want to delete these resources in given sandbox ?")
        .accept("y")
        .accept("n")
        .until_acceptable()
        .show_defaults()
        .confirm();
    answer == Answer::YES
}

fn count_of(grouped: &HashMap<Entity, Vec<String>>) -> String {
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

pub async fn prune(auth: &Auth, sandbox: &str) {
    let client = aws::resourcetag::make_client(auth).await;
    let arns = aws::resourcetag::get_resources(&client, "sandbox", sandbox).await;

    if arns.len() == 0 {
        println!("No resources found for given sandbox");
        std::process::exit(1);
    }

    let grouped = group_entities(arns);
    println!("{}", count_of(&grouped));
    let cont = maybe_continue();
    if !cont {
        std::process::exit(1);
    }

    for (entity, arns) in grouped {
        match entity {
            Entity::Function => {
                let client = aws::lambda::make_client(auth).await;
                for arn in arns {
                    aws::lambda::delete(&client, &arn).await;
                }
            }
            Entity::State => {
                let client = aws::sfn::make_client(auth).await;
                for arn in arns {
                    aws::sfn::delete(&client, &arn).await;
                }
            }

            Entity::Mutation => {
                let client = aws::appsync::make_client(auth).await;
                for arn in arns {
                    let api_id = kit::split_last(&arn, "/");
                    aws::appsync::delete(&client, &api_id).await;
                }
            }
            _ => println!("Skipping entity {}", &entity.to_str()),
        }
    }
}

pub async fn list(auth: &Auth, sandbox: &str) {
    let client = aws::resourcetag::make_client(auth).await;
    let mut arns = aws::resourcetag::get_resources(&client, "sandbox", sandbox).await;
    arns.sort();
    for arn in &arns {
        println!("{}", &arn)
    }

    let grouped = group_entities(arns);
    println!("");
    println!("{}", count_of(&grouped));
}
