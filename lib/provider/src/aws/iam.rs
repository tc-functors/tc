use super::constants;
use crate::Auth;
use aws_sdk_iam::{
    Client,
    Error,
    config,
    config::retry::{
        RetryConfig,
        RetryMode,
    },
};
use colored::Colorize;
use kit as u;
use kit::LogUpdate;
use std::io::stdout;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        config::Builder::from(shared_config)
            .behavior_version(constants::behavior_version())
            .timeout_config(constants::timeout_config())
            .retry_config(
                RetryConfig::standard()
                    .with_retry_mode(RetryMode::Adaptive)
                    .with_max_attempts(constants::MAX_ATTEMPTS)
                    .with_initial_backoff(constants::INITIAL_BACKOFF)
                    .with_max_backoff(constants::MAX_BACKOFF),
            )
            .build(),
    )
}

#[derive(Debug)]
pub struct Role {
    pub name: String,
    pub policy_name: String,
    pub policy_arn: String,
    pub trust_policy: String,
    pub policy_doc: String,
}

impl Role {
    async fn create(&self, client: &Client) {
        let mut log_update = LogUpdate::new(stdout()).unwrap();

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "creating policy".cyan()
        ));
        self.find_or_create_policy(client).await;

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "attachable".cyan()
        ));
        self.wait_until_attachable(client).await;

        let _ = log_update.render(&format!("Creating role {} ({})", self.name, "role".cyan()));
        self.find_or_create_role(client).await;

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "attaching".cyan()
        ));
        self.attach_policy(client).await;
        self.wait_until_attached(client).await;
        // FIXME: iam is eventually consistent. There is no way to know if the role is really
        // useable
        u::sleep(4000);

        let _ = log_update.render(&format!(
            "Creating role {} ({})",
            self.name,
            "attached".green()
        ));
    }

    pub async fn delete(&self, client: &Client) -> Result<(), Error> {
        println!("Deleting role {}", self.name);
        self.detach_policy(client).await?;
        self.wait_until_detached(client).await;
        self.delete_non_default_versions(client).await?;
        self.delete_policy(client).await?;
        self.delete_role(client).await?;
        Ok(())
    }

    async fn update(&self, client: &Client) -> Result<(), Error> {
        let mut log_update = LogUpdate::new(stdout()).unwrap();

        let _ = log_update.render(&format!(
            "Updating role {} ({})",
            self.name,
            "pruning old versions".blue()
        ));
        self.delete_non_default_versions(client).await?;

        let _ = log_update.render(&format!(
            "Updating role {} ({})",
            self.name,
            "creating policy version".blue()
        ));
        client
            .create_policy_version()
            .policy_arn(&self.policy_arn)
            .policy_document(&self.policy_doc)
            .set_as_default(true)
            .send()
            .await
            .unwrap();

        self.find_or_create_role(client).await;

        let _ = log_update.render(&format!(
            "Updating role {} ({})",
            self.name,
            "updated".green()
        ));
        Ok(())
    }

    pub async fn create_or_update(&self, client: &Client) -> Result<(), Error> {
        let res = client.get_role().role_name(&self.name).send().await;
        match res {
            Ok(_) => self.update(client).await?,
            Err(_) => self.create(client).await,
        }
        Ok(())
    }

    pub async fn find_or_create(&self, client: &Client) {
        let res = client.get_role().role_name(&self.name).send().await;
        match res {
            Ok(_) => (),
            Err(_) => self.create(client).await,
        };
    }

    pub async fn create_policy(&self, client: &Client) -> String {
        let res = client
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

    async fn find_policy(&self, client: &Client) -> Result<Option<String>, Error> {
        let res = client
            .get_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        match res {
            Ok(r) => Ok(r.policy.unwrap().arn),
            Err(_) => Ok(None),
        }
    }

    pub async fn find_or_create_policy(&self, client: &Client) -> String {
        let res = self.find_policy(client).await.unwrap();
        match res {
            Some(a) => a,
            None => self.create_policy(client).await,
        }
    }

    async fn find_role(&self, client: &Client) -> Result<Option<String>, Error> {
        let res = client.get_role().role_name(&self.name).send().await;
        match res {
            Ok(r) => Ok(Some(r.role.unwrap().arn)),
            Err(_) => Ok(None),
        }
    }

    async fn create_role(&self, client: &Client) -> String {
        let res = client
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

    pub async fn find_or_create_role(&self, client: &Client) -> String {
        let arn = self.find_role(client).await.unwrap();
        match arn {
            Some(a) => a,
            None => self.create_role(client).await,
        }
    }

    pub async fn attach_policy(&self, client: &Client) {
        client
            .attach_role_policy()
            .role_name(&self.name)
            .policy_arn(&self.policy_arn)
            .send()
            .await
            .unwrap();
    }

    pub async fn detach_policy(&self, client: &Client) -> Result<(), Error> {
        let res = client
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

    pub async fn delete_policy(&self, client: &Client) -> Result<(), Error> {
        let res = client
            .delete_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    pub async fn delete_role(&self, client: &Client) -> Result<(), Error> {
        let res = client.delete_role().role_name(&self.name).send().await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Ok(()),
        }
    }

    async fn list_policy_versions(&self, client: &Client) -> Vec<(String, bool)> {
        let res = client
            .list_policy_versions()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        match res {
            Ok(r) => r
                .versions()
                .iter()
                .filter_map(|v| {
                    v.version_id()
                        .map(|id| (id.to_string(), v.is_default_version()))
                })
                .collect(),
            Err(_) => vec![],
        }
    }

    async fn delete_non_default_versions(&self, client: &Client) -> Result<(), Error> {
        let versions = self.list_policy_versions(client).await;
        for (version_id, is_default) in versions {
            if !is_default {
                let _ = client
                    .delete_policy_version()
                    .policy_arn(&self.policy_arn)
                    .version_id(version_id)
                    .send()
                    .await;
            }
        }
        Ok(())
    }

    pub async fn is_policy_attachable(&self, client: &Client) -> bool {
        let res = client
            .get_policy()
            .policy_arn(&self.policy_arn)
            .send()
            .await;
        res.unwrap().policy.unwrap().is_attachable
    }

    pub async fn is_policy_attached(&self, client: &Client) -> bool {
        let res = client
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

    pub async fn wait_until_attachable(&self, client: &Client) {
        let mut ready = false;
        while !ready {
            ready = self.is_policy_attachable(client).await;
            u::sleep(1000)
        }
    }

    pub async fn wait_until_attached(&self, client: &Client) {
        let mut ready = false;
        while !ready {
            ready = self.is_policy_attached(client).await;
            u::sleep(2000)
        }
    }

    pub async fn wait_until_detached(&self, client: &Client) {
        let mut ready = false;
        while !ready {
            ready = !self.is_policy_attached(client).await;
            u::sleep(2000)
        }
    }
}

pub async fn find_policy_doc(
    client: &Client,
    _role_name: &str,
    policy_arn: &str,
) -> Option<String> {
    let res = client.get_policy().policy_arn(policy_arn).send().await;

    match res {
        Ok(r) => {
            if let Some(version) = r.policy.unwrap().default_version_id {
                let dres = client
                    .get_policy_version()
                    .policy_arn(policy_arn)
                    .version_id(version)
                    .send()
                    .await;
                match dres {
                    Ok(r) => {
                        let doc = r.policy_version.unwrap().document.unwrap();
                        Some(urlencoding::decode(&doc).expect("UTF-8").to_string())
                    }
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}


async fn list_roles_by_token(client: &Client, token: &str) -> (Vec<(String, String)>, Option<String>, bool) {
    let res = client
        .list_roles()
        .marker(token)
        .send()
        .await
        .unwrap();
    let roles = res.roles.to_vec();
    let mut xs: Vec<(String, String)> = vec![];
    for role in roles {
        xs.push((role.role_name, role.arn))
    }
    (xs, res.marker, res.is_truncated)
}

pub async fn list_roles(client: &Client) -> Vec<(String, String)> {
    let res = client
        .list_roles()
        .send()
        .await
        .unwrap();
    let mut token: Option<String> = res.marker;
    let mut is_truncated = res.is_truncated;

    let roles = res.roles.to_vec();
    let mut xs: Vec<(String, String)> = vec![];
    for role in roles {
        xs.push((role.role_name, role.arn))
    }


    match token {
        Some(tk) => {
            token = Some(tk);
            while is_truncated {
                let (x, t, truncated) = list_roles_by_token(client, &token.unwrap()).await;
                xs.extend(x);
                token = t.clone();
                is_truncated = truncated;
                if let Some(x) = t {
                    if x.is_empty() {
                        break;
                    }
                }

            }
        },
        None => (),
    }
    xs

}

pub type IamClient = Client;
