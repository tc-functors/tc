use provider::{
    Auth,
    aws::{
        iam,
        iam::Role,
        s3,
        s3files,
    },
};

fn make_policy(bucket: &str, account: &str, region: &str) -> String {
    format!(
        r#"{{
    "Version": "2012-10-17",
    "Statement": [
        {{
            "Sid": "S3BucketPermissions",
            "Effect": "Allow",
            "Action": [
                "s3:ListBucket*"
            ],
            "Resource": "arn:aws:s3:::{bucket}",
            "Condition": {{
                "StringEquals": {{
                    "aws:ResourceAccount": "{account}"
                }}
            }}
        }},
        {{
            "Sid": "S3ObjectPermissions",
            "Effect": "Allow",
            "Action": [
                "s3:AbortMultipartUpload",
                "s3:DeleteObject*",
                "s3:GetObject*",
                "s3:List*",
                "s3:PutObject*"
            ],
            "Resource": "arn:aws:s3:::{bucket}/*",
            "Condition": {{
                "StringEquals": {{
                    "aws:ResourceAccount": "{account}"
                }}
            }}
        }},
        {{
            "Sid": "UseKmsKeyWithS3Files",
            "Effect": "Allow",
            "Action": [
                "kms:GenerateDataKey",
                "kms:Encrypt",
                "kms:Decrypt",
                "kms:ReEncryptFrom",
                "kms:ReEncryptTo"
            ],
            "Condition": {{
                "StringLike": {{
                    "kms:ViaService": "s3.{region}.amazonaws.com",
                    "kms:EncryptionContext:aws:s3:arn": [
                        "arn:aws:s3:::{bucket}",
                        "arn:aws:s3:::{bucket}/*"
                    ]
                }}
            }},
            "Resource": "arn:aws:kms:{region}:{account}:*"
        }},
        {{
            "Sid": "EventBridgeManage",
            "Effect": "Allow",
            "Action": [
                "events:DeleteRule",
                "events:DisableRule",
                "events:EnableRule",
                "events:PutRule",
                "events:PutTargets",
                "events:RemoveTargets"
            ],
            "Condition": {{
                "StringEquals": {{
                    "events:ManagedBy": "elasticfilesystem.amazonaws.com"
                }}
            }},
            "Resource": [
                "arn:aws:events:*:*:rule/DO-NOT-DELETE-S3-Files*"
            ]
        }},
        {{
            "Sid": "EventBridgeRead",
            "Effect": "Allow",
            "Action": [
                "events:DescribeRule",
                "events:ListRuleNamesByTarget",
                "events:ListRules",
                "events:ListTargetsByRule"
            ],
            "Resource": [
                "arn:aws:events:*:*:rule/*"
            ]
        }}
    ]
}}"#
    )
}

fn make_trust_policy(account: &str, region: &str) -> String {
    format!(
        r#"{{
    "Version": "2012-10-17",
    "Statement": [
        {{
            "Sid": "AllowS3FilesAssumeRole",
            "Effect": "Allow",
            "Principal": {{
                "Service": "elasticfilesystem.amazonaws.com"
            }},
            "Action": "sts:AssumeRole",
            "Condition": {{
                "StringEquals": {{
                    "aws:SourceAccount": "{account}"
                }},
                "ArnLike": {{
                    "aws:SourceArn": "arn:aws:s3files:{region}:{account}:file-system/*"
                }}
            }}
        }}
    ]
}}"#
    )
}

async fn create_role(auth: &Auth, name: &str, bucket: &str) -> String {
    let client = iam::make_client(auth).await;
    let policy = make_policy(bucket, &auth.account, &auth.region);
    let trust_policy = make_trust_policy(&auth.account, &auth.region);
    let policy_arn = auth.policy_arn(name);
    let role_arn = auth.role_arn(name);
    let role = Role {
        name: name.to_string(),
        trust_policy: trust_policy,
        policy_arn: policy_arn,
        policy_name: name.to_string(),
        policy_doc: policy,
    };
    let _ = role.create_or_update(&client).await;
    role_arn
}

pub struct FsOpts {
    pub bucket: String,
    pub role_arn: Option<String>,
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
}

pub async fn create_s3_fs(auth: &Auth, opts: FsOpts) -> String {
    let FsOpts {
        bucket,
        subnets,
        security_groups,
        ..
    } = opts;
    println!("Creating store: {}", &bucket);

    let role_arn = if let Some(role_arn) = opts.role_arn {
        role_arn
    } else {
        let name = format!("tc-store-{}-{}", &bucket, &auth.region);
        tracing::debug!("Creating fs role: {}", &name);
        create_role(auth, &name, &bucket).await
    };

    match std::env::var("TC_CREATE_FS_BUCKET") {
        Ok(_) => {
            let s3_client = s3::make_client(auth).await;
            s3::find_or_create_bucket(&s3_client, &bucket, &auth.region).await;
            tracing::debug!("Enabling bucket versioning...");
            s3::enable_versioning(&s3_client, &bucket).await;
        }
        Err(_) => (),
    }

    let s3f_client = s3files::make_client(auth).await;
    println!("Creating fs with bucket {}", &bucket);
    let fs_id = s3files::find_or_create_fs(&s3f_client, &auth.s3_arn(&bucket), &role_arn).await;

    tracing::debug!("Creating access point fs: {}...", &fs_id);
    let ap_arn = s3files::find_or_create_ap(&s3f_client, &fs_id).await;

    for subnet_id in subnets {
        tracing::debug!("Creating mount target {}...", &subnet_id);
        let _ =
            s3files::find_or_create_mt(&s3f_client, &fs_id, &subnet_id, security_groups.clone())
                .await;
    }

    ap_arn
}
