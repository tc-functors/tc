use aws_sdk_iam::config as iam_config;
use aws_sdk_iam::config::retry::RetryConfig;
use aws_sdk_iam::{Client, Error};

use super::Env;
use kit::*;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::from_conf(
        iam_config::Builder::from(&shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

#[derive(Debug)]
pub struct Role {
    pub client: Client,
    pub name: String,
    pub policy_name: String,
    pub policy_arn: String,
    pub trust_policy: String,
    pub policy_doc: String,
}

impl Role {
    pub async fn create(&self) {
        println!("Creating role {}", self.name);
        self.find_or_create_policy().await;
        self.find_or_create_role().await;
        self.attach_policy().await;
        sleep(4000);
    }

    pub async fn delete(&self) -> Result<(), Error> {
        println!("Deleting role {}", self.name);
        self.detach_policy().await?;
        self.delete_policy().await?;
        self.delete_role().await?;
        Ok(())
    }

    pub async fn update(&self) -> Result<(), Error> {
        println!("Updating role {}", self.name);
        self.detach_policy().await?;
        self.delete_policy().await?;
        sleep(2000);
        self.find_or_create_policy().await;
        self.attach_policy().await;
        self.find_or_create_role().await;
        Ok(())
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
        sleep(2000);
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
        sleep(2000);
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
}
