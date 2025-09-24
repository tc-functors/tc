use crate::Auth;
use aws_sdk_acm::Client;
use aws_sdk_acm::types::CertificateStatus;
use aws_sdk_acm::types::ValidationMethod;
use aws_sdk_acm::types::ResourceRecord;
use std::collections::HashMap;
use kit as u;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.get_global_config().await;
    Client::new(shared_config)
}

async fn list_certs_by_token(
    client: &Client,
    token: &str,
) -> (HashMap<String, String>, Option<String>) {
    let res = client
        .list_certificates()
        .next_token(token.to_string())
        .max_items(50)
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, String> = HashMap::new();
    let summaries = res.certificate_summary_list.unwrap();
    for summary in summaries {
        if let (Some(domain), Some(arn)) = (summary.domain_name, summary.certificate_arn) {
            h.insert(domain, arn);
        }
    }
    (h, res.next_token)
}

async fn list_certs(client: &Client) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let r = client.list_certificates().max_items(50).send().await;
    match r {
        Ok(res) => {
            let mut token: Option<String> = res.next_token;

            let summaries = res.certificate_summary_list.unwrap();
            for summary in summaries {
                if let (Some(domain), Some(arn)) = (summary.domain_name, summary.certificate_arn) {
                    h.insert(domain, arn);
                }
            }

            match token {
                Some(tk) => {
                    token = Some(tk);
                    while token.is_some() {
                        let (xs, t) = list_certs_by_token(client, &token.unwrap()).await;
                        h.extend(xs.clone());
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
        }
        Err(e) => panic!("{}", e),
    }
    h
}

pub async fn find_cert(client: &Client, domain: &str) -> Option<String> {
    let certs = list_certs(client).await;
    certs.get(domain).cloned()
}

pub async fn request_cert(client: &Client, domain: &str, token: &str) -> String {
    let res = client
        .request_certificate()
        .domain_name(domain)
        .validation_method(ValidationMethod::Dns)
        .idempotency_token(token)
        .send()
        .await;

    res.unwrap().certificate_arn.unwrap()
}

async fn get_status(client: &Client, cert_arn: &str) -> CertificateStatus {
    let res = client
        .describe_certificate()
        .certificate_arn(cert_arn)
        .send()
        .await;
    let status = res.unwrap().certificate.unwrap().status.unwrap();
    status
}

pub async fn wait_until_validated(client: &Client, arn: &str) {
    let st = get_status(client, arn).await;
    let mut status: CertificateStatus = st;
    while status == CertificateStatus::PendingValidation {
        let cur = get_status(client, arn).await;
        status = cur.clone();
        println!("Waiting for cert to be validated: {:?}...", status);
        u::sleep(10000);
    }
}

pub async fn get_domain_validation_records(client: &Client, arn: &str) ->  Vec<ResourceRecord> {
    let r = client
        .describe_certificate()
        .certificate_arn(arn)
        .send()
        .await;
    let res = r.unwrap().certificate.unwrap();
    if let Some(status) = res.status {
        match status {
            CertificateStatus::Issued => vec![],
            _ => {
                let options = res.domain_validation_options.unwrap();
                let xs: Vec<ResourceRecord> = options.into_iter().map(|opt|  opt.resource_record.unwrap()).collect();
                xs
            }
        }
    } else {
        vec![]
    }
}
