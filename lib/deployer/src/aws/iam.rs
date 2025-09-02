use authorizer::Auth;
use aws_sdk_iam::{
    Client,
    Error,
    config as iam_config,
    config::retry::RetryConfig,
    types::{
        Tag,
        builders::TagBuilder,
    },
};
use colored::Colorize;
use kit as u;
use kit::LogUpdate;
use std::{
    collections::HashMap,
    io::stdout,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        iam_config::Builder::from(shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

fn make_tag(key: String, value: String) -> Tag {
    let tb = TagBuilder::default();
    tb.key(key).value(value).build().unwrap()
}

pub fn make_tags(kvs: HashMap<String, String>) -> Vec<Tag> {
    let mut tags: Vec<Tag> = vec![];
    for (k, v) in kvs.into_iter() {
        let tag = make_tag(k, v);
        tags.push(tag);
    }
    tags
}

#[derive(Debug)]
pub struct Role {
    pub client: Client,
    pub name: String,
    pub policy_name: String,
    pub policy_arn: String,
    pub trust_policy: String,
    pub policy_doc: String,
    pub tags: Option<Vec<Tag>>,
}

impl Role {
    async fn create(&self) {
        let mut log_update = LogUpdate::new(stdout()).unwrap();

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "creating policy".cyan()
        ));
        self.find_or_create_policy().await;

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "attachable".cyan()
        ));
        self.wait_until_attachable().await;

        let _ = log_update.render(&format!("Creating role {} ({})", self.name, "role".cyan()));
        self.find_or_create_role().await;

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "attaching".cyan()
        ));
        self.attach_policy().await;
        self.wait_until_attached().await;
        // FIXME: iam is eventually consistent. There is no way to know if the role is really
        // useable
        u::sleep(4000);

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "attached".green()
        ));
    }

    pub async fn delete(&self) -> Result<(), Error> {
        println!("Deleting role {}", self.name);
        self.detach_policy().await?;
        self.wait_until_detached().await;
        self.delete_policy().await?;
        self.delete_role().await?;
        Ok(())
    }

    async fn update(&self) -> Result<(), Error> {
        let mut log_update = LogUpdate::new(stdout()).unwrap();

        let _ = log_update.render(&format!(
            "Updating role {} ({})",
            self.name,
            "detaching policy".blue()
        ));
        self.detach_policy().await?;

        self.wait_until_detached().await;

        let _ = log_update.render(&format!("Updating role {} (deleting policy)", self.name));
        self.delete_policy().await?;

        u::sleep(2000);

        let _ = log_update.render(&format!("Updating role {} (creating policy)", self.name));
        self.find_or_create_policy().await;
        self.wait_until_attachable().await;

        let _ = log_update.render(&format!("Updating role {} (attaching policy)", self.name));
        self.attach_policy().await;
        self.find_or_create_role().await;
        self.wait_until_attached().await;
        let _ = log_update.render(&format!("Updating role {} (attached)", self.name));
        Ok(())
    }

    pub async fn create_or_update(&self) -> Result<(), Error> {
        let res = self.client.get_role().role_name(&self.name).send().await;
        match res {
            Ok(_) => self.update().await?,
            Err(_) => self.create().await,
        }
        Ok(())
    }

    pub async fn find_or_create(&self) {
        let res = self.client.get_role().role_name(&self.name).send().await;
        match res {
            Ok(_) => (),
            Err(_) => self.create().await,
        };
    }

    pub async fn create_policy(&self) -> String {
        let res = self
            .client
            .create_policy()
            .policy_name(&self.policy_name)
            .policy_document(&self.policy_doc)
            .send()
            .await
            .unwrap();
        match res.policy {
            Some(p) => p.arn.unwrap(),
            None => panic!("Error creating policy"),
        }
    }

    async fn find_policy(&self) -> Result<Option<String>, Error> {
        let res = self
            .client
            .get_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        match res {
            Ok(r) => Ok(r.policy.unwrap().arn),
            Err(_) => Ok(None),
        }
    }

    pub async fn find_or_create_policy(&self) -> String {
        let res = self.find_policy().await.unwrap();
        match res {
            Some(a) => a,
            None => self.create_policy().await,
        }
    }

    async fn find_role(&self) -> Result<Option<String>, Error> {
        let res = self.client.get_role().role_name(&self.name).send().await;
        match res {
            Ok(r) => Ok(Some(r.role.unwrap().arn)),
            Err(_) => Ok(None),
        }
    }

    async fn create_role(&self) -> String {
        let res = self
            .client
            .create_role()
            .role_name(&self.name)
            .assume_role_policy_document(&self.trust_policy)
            .send()
            .await
            .unwrap();
        match res.role {
            Some(r) => r.arn,
            None => panic!("Error creating policy"),
        }
    }

    pub async fn find_or_create_role(&self) -> String {
        let arn = self.find_role().await.unwrap();
        match arn {
            Some(a) => a,
            None => self.create_role().await,
        }
    }

    pub async fn attach_policy(&self) {
        self.client
            .attach_role_policy()
            .role_name(&self.name)
            .policy_arn(&self.policy_arn)
            .send()
            .await
            .unwrap();
    }

    pub async fn detach_policy(&self) -> Result<(), Error> {
        let res = self
            .client
            .detach_role_policy()
            .role_name(&self.name)
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn delete_policy(&self) -> Result<(), Error> {
        let res = self
            .client
            .delete_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn delete_role(&self) -> Result<(), Error> {
        let res = self.client.delete_role().role_name(&self.name).send().await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn is_policy_attachable(&self) -> bool {
        let res = self
            .client
            .get_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        res.unwrap().policy.unwrap().is_attachable
    }

    pub async fn is_policy_attached(&self) -> bool {
        let res = self
            .client
            .get_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;

        match res {
            Ok(r) => {
                let c = r.policy.unwrap().attachment_count.unwrap();
                c > 0
            }
            Err(_) => false,
        }
    }

    pub async fn wait_until_attachable(&self) {
        let mut ready = false;
        while !ready {
            ready = self.is_policy_attachable().await;
            u::sleep(1000)
        }
    }

    pub async fn wait_until_attached(&self) {
        let mut ready = false;
        while !ready {
            ready = self.is_policy_attached().await;
            u::sleep(2000)
        }
    }

    pub async fn wait_until_detached(&self) {
        let mut ready = false;
        while !ready {
            ready = !self.is_policy_attached().await;
            u::sleep(2000)
        }
    }
}
