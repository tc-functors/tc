use crate::aws::{
    iam,
    iam::Role,
};
use authorizer::Auth;

// TODO defaults. Move to compiler

fn base_trust_policy() -> String {
    format!(
        r#"{{"Version": "2012-10-17",
    "Statement": [
        {{
            "Effect": "Allow",
            "Principal": {{
                "Service": [
                    "lambda.amazonaws.com",
                    "events.amazonaws.com",
                    "states.amazonaws.com",
                    "logs.amazonaws.com",
                    "apigateway.amazonaws.com",
                    "appsync.amazonaws.com",
                    "scheduler.amazonaws.com"
                ]
            }},
            "Action": "sts:AssumeRole"
        }}
    ]
     }}"#
    )
}

fn base_lambda_policy() -> String {
    format!(
        r#"{{"Statement": [
    {{
      "Action": "lambda:InvokeFunction",
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "TcBasicLambdaInvoke"
    }},
    {{
      "Action": "states:*",
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "SFNInvoke1"
    }},
    {{
      "Action": [
        "events:PutTargets",
        "events:PutRule",
        "events:DescribeRule",
        "events:PutEvents"
      ],
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "SFNEvents1"
    }},
    {{
      "Action": [
        "xray:PutTraceSegments",
        "xray:PutTelemetryRecords",
        "xray:GetSamplingTargets",
        "xray:GetSamplingStatisticSummaries",
        "xray:GetSamplingRules",
        "ssm:GetParameters",
        "ssm:GetParameter",
        "logs:CreateLogGroup",
	"logs:PutLogEvents",
        "logs:CreateLogDelivery",
        "logs:CreateLogStream",
        "logs:GetLogDelivery",
        "logs:UpdateLogDelivery",
        "logs:DeleteLogDelivery",
        "logs:ListLogDeliveries",
        "logs:PutResourcePolicy",
        "logs:DescribeResourcePolicies",
        "logs:DescribeLogStreams",
        "logs:DescribeLogGroups",
        "logs:CreateLogStream",
        "logs:CreateLogGroup",
        "logs:CreateLogGroup"
      ],
    "Effect": "Allow",
    "Resource": "*",
    "Sid": "AccessToCloudWatch1"
  }},
  {{
      "Effect": "Allow",
      "Action": [
        "ec2:CreateNetworkInterface",
        "ec2:DescribeNetworkInterfaces",
        "ec2:DeleteNetworkInterface",
        "ec2:AssignPrivateIpAddresses",
        "ec2:UnassignPrivateIpAddresses"
      ],
      "Resource": "*"
  }},

  {{
      "Effect": "Allow",
      "Action": [
        "elasticfilesystem:ClientMount",
        "elasticfilesystem:ClientRootAccess",
        "elasticfilesystem:ClientWrite",
        "elasticfilesystem:DescribeMountTargets"
      ],
      "Resource": "*"
  }},

  {{
      "Effect": "Allow",
      "Action": [
        "kms:Decrypt"
      ],
      "Resource": "*"
  }}

  ],
  "Version": "2012-10-17"
}}"#
    )
}

fn base_sfn_policy() -> String {
    format!(
        r#"{{"Statement": [
    {{
      "Action": "lambda:InvokeFunction",
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "LambdaInvoke1"
    }},
    {{
      "Action": "states:*",
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "SFNInvoke1"
    }},
    {{
      "Action": [
        "events:PutTargets",
        "events:PutRule",
        "events:DescribeRule",
        "events:PutEvents"
      ],
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "SFNEvents1"
    }},
    {{
      "Action": [
        "xray:PutTraceSegments",
        "xray:PutTelemetryRecords",
        "xray:GetSamplingTargets",
        "xray:GetSamplingStatisticSummaries",
        "xray:GetSamplingRules",
        "ssm:GetParameters",
        "logs:CreateLogGroup",
	"logs:PutLogEvents",
        "logs:CreateLogDelivery",
        "logs:CreateLogStream",
        "logs:GetLogDelivery",
        "logs:UpdateLogDelivery",
        "logs:DeleteLogDelivery",
        "logs:ListLogDeliveries",
        "logs:PutResourcePolicy",
        "logs:DescribeResourcePolicies",
        "logs:DescribeLogStreams",
        "logs:DescribeLogGroups",
        "logs:CreateLogStream",
        "logs:CreateLogGroup",
        "logs:CreateLogGroup"
      ],
    "Effect": "Allow",
    "Resource": "*",
    "Sid": "AccessToCloudWatch1"
  }}
  ],
  "Version": "2012-10-17"
}}"#
    )
}

fn base_api_policy() -> String {
    format!(
        r#"{{"Statement": [
    {{
      "Action": "lambda:InvokeFunction",
      "Effect": "Allow",
      "Resource": "*"
    }},
    {{
      "Action": "states:*",
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "SFNInvoke1"
    }}
  ],
  "Version": "2012-10-17"
}}"#
    )
}

fn base_event_policy(region: &str, account: &str) -> String {
    format!(
        r#"{{"Statement": [
    {{
      "Action": [
        "events:PutTargets",
        "events:PutRule",
        "events:DescribeRule",
        "events:PutEvents"
      ],
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "SFNEvents1"
    }},
    {{
         "Effect": "Allow",
            "Action": [
                "lambda:InvokeFunction"
            ],
            "Resource": [
                "arn:aws:lambda:{region}:{account}:function:*",
                "arn:aws:lambda:{region}:{account}:function:*:*"
            ],
      "Sid": "invokelambda1"
    }},
    {{
      "Action": [
        "states:StartExecution"
      ],
      "Effect": "Allow",
      "Resource": "*",
      "Sid": "StartsEvent"
    }},
    {{
      "Effect": "Allow",
            "Action": [
                "appsync:GraphQL"
            ],
            "Resource": [
                "arn:aws:appsync:{region}:{account}:apis/*/types/Mutation/fields/*"
            ],
      "Sid": "Graphqlq"
    }}

  ],
  "Version": "2012-10-17"
}}"#
    )
}

fn base_appsync_policy(region: &str, account: &str) -> String {
    format!(
        r#"{{"Statement": [
    {{
         "Effect": "Allow",
            "Action": [
                "lambda:invokeFunction"
            ],
            "Resource": [
                "arn:aws:lambda:{region}:{account}:function:*",
                "arn:aws:lambda:{region}:{account}:function:*:*"
            ],
      "Sid": "Appsync1"
    }},
    {{
      "Effect": "Allow",
            "Action": [
                "appsync:GraphQL"
            ],
            "Resource": [
                "arn:aws:appsync:{region}:{account}:apis/*/types/Mutation/fields/*"
            ],
      "Sid": "Graphqlq"
    }}

  ],
  "Version": "2012-10-17"
}}"#
    )
}

fn base_role_name(name: &str) -> String {
    format!("tc-base-{}-role", name)
}

fn base_policy_name(name: &str) -> String {
    format!("tc-base-{}-policy", name)
}

fn policy_arn(account: &str, name: &str) -> String {
    format!("arn:aws:iam::{}:policy/{}", account, name)
}

async fn make_role(auth: &Auth, name: &str) -> Role {
    let policy_doc = match name {
        "lambda" => base_lambda_policy(),
        "sfn" => base_sfn_policy(),
        "event" => base_event_policy(&auth.region, &auth.account),
        "api" => base_api_policy(),
        "appsync" => base_appsync_policy(&auth.region, &auth.account),
        _ => panic!("No such policy"),
    };
    let client = iam::make_client(auth).await;
    let role_fqn = base_role_name(name);
    let policy_fqn = base_policy_name(name);
    let policy_arn = policy_arn(&auth.account, &policy_fqn);
    Role {
        client: client.clone(),
        name: role_fqn,
        trust_policy: base_trust_policy(),
        policy_arn: policy_arn,
        policy_name: policy_fqn,
        policy_doc: policy_doc,
    }
}

pub async fn create_role(auth: &Auth, name: &str) {
    let role = make_role(auth, name).await;
    let _ = role.create_or_update().await;
}

pub async fn delete_role(auth: &Auth, name: &str) {
    let role = make_role(auth, name).await;
    let _ = role.delete().await;
}

pub async fn create_roles(auth: &Auth) {
    let roles = vec!["lambda", "sfn", "event", "api", "appsync"];
    for role in roles {
        create_role(auth, role).await;
    }
}
