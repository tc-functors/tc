pub fn trust_policy() -> String {
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

pub fn lambda_policy() -> String {
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

pub fn sfn_policy() -> String {
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

pub fn api_policy() -> String {
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

pub fn event_policy(region: &str, account: &str) -> String {
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

pub fn appsync_policy(region: &str, account: &str) -> String {
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
