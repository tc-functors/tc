use crate::Entity;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_with::{
    OneOrMany,
    formats::PreferOne,
    serde_as,
};

fn default_sid() -> Option<String> {
    Some(format!("TcBaseDefault{}", randstr()))
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Action {
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    #[serde(rename(serialize = "Action", deserialize = "Action"))]
    action: Vec<String>,
    #[serde(rename(serialize = "Effect", deserialize = "Effect"))]
    effect: String,
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    #[serde(rename(serialize = "Resource", deserialize = "Resource"))]
    resource: Vec<String>,
    #[serde(
        rename(serialize = "Sid", deserialize = "Sid"),
        default = "default_sid"
    )]
    sid: Option<String>,
}

fn make_sid(ec: &str) -> Option<String> {
    Some(format!("TcBase{}{{{{sandbox_trimmed}}}}", ec))
}

fn make_lambda_actions() -> Vec<Action> {
    vec![
        Action {
            action: v!["lambda:InvokeFunction"],
            effect: s!("Allow"),
            resource: v![
                "arn:aws:lambda:{{region}}:{{account}}:function:*",
                "arn:aws:lambda:{{region}}:{{account}}:function:*:*"
            ],
            sid: make_sid("LambdaFunction"),
        },
        Action {
            action: v!["states:*"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("LambdaState"),
        },
        Action {
            action: v![
                "events:PutTargets",
                "events:PutRule",
                "events:DescribeRule",
                "events:PutEvents"
            ],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("LambdaEvent"),
        },
        Action {
            action: v![
                "logs:CreateLogGroup",
                "logs:PutLogEvents",
                "logs:CreateLogDelivery",
                "logs:CreateLogStream"
            ],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("LambdaLog"),
        },
        Action {
            action: v!["s3:GetObject", "s3:GetObjectVersion"],
            effect: s!("Allow"),
            resource: v![&format!("arn:aws:s3:::{{{{ASSET_BUCKET}}}}/*")],
            sid: make_sid("LambdaAssetAccess1"),
        },
        Action {
            action: v!["ssm:GetParameters", "ssm:GetParameter"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("LambdaSSM"),
        },
        Action {
            action: v![
                "xray:PutTraceSegments",
                "xray:PutTelemetryRecords",
                "xray:GetSamplingTargets",
                "xray:GetSamplingStatisticSummaries",
                "xray:GetSamplingRules"
            ],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("LambdaXray"),
        },
        Action {
            action: v!["kms:Decrypt"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("LambdaKMS"),
        },
    ]
}

fn make_microvm_actions() -> Vec<Action> {
    vec![
        Action {
            action: v!["s3:GetObject"],
            effect: s!("Allow"),
            resource: v![&format!("arn:aws:s3:::{{{{ASSET_BUCKET}}}}/*")],
            sid: make_sid("MVS3"),
        },
        Action {
            action: v![
                "logs:CreateLogGroup",
                "logs:PutLogEvents",
                "logs:CreateLogStream"
            ],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("MVLambdaLog"),
        },
    ]
}

fn make_sfn_actions() -> Vec<Action> {
    vec![
        Action {
            action: v!["lambda:InvokeFunction"],
            effect: s!("Allow"),
            resource: v![
                "arn:aws:lambda:{{region}}:{{account}}:function:*",
                "arn:aws:lambda:{{region}}:{{account}}:function:*:*"
            ],
            sid: make_sid("StateLambda"),
        },
        Action {
            action: v!["states:DescribeExecution", "states:StopExecution"],
            effect: s!("Allow"),
            resource: v!["arn:aws:states:{{region}}:{{account}}:stateMachine:*"],
            sid: make_sid("StateState"),
        },
        Action {
            action: v!["states:StartExecution"],
            effect: s!("Allow"),
            resource: v!["arn:aws:states:{{region}}:{{account}}:stateMachine:*"],
            sid: make_sid("StateStateChild"),
        },
        Action {
            action: v![
                "events:PutTargets",
                "events:PutRule",
                "events:DescribeRule",
                "events:PutEvents"
            ],
            effect: s!("Allow"),
            resource: v!["arn:aws:events:{{region}}:{{account}}:rule/StepFunctions*"],
            sid: make_sid("StateEvent"),
        },
        Action {
            action: v![
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
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("StateLogs"),
        },
        Action {
            action: v!["ssm:GetParameters", "ssm:GetParameter"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("StateSSM"),
        },
        Action {
            action: v![
                "xray:PutTraceSegments",
                "xray:PutTelemetryRecords",
                "xray:GetSamplingTargets",
                "xray:GetSamplingStatisticSummaries",
                "xray:GetSamplingRules"
            ],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("StateXray"),
        },
    ]
}

fn make_api_actions() -> Vec<Action> {
    vec![
        Action {
            action: v!["lambda:InvokeFunction"],
            effect: s!("Allow"),
            resource: v![
                "arn:aws:lambda:{{region}}:{{account}}:function:*",
                "arn:aws:lambda:{{region}}:{{account}}:function:*:*"
            ],
            sid: make_sid("ApiLambda"),
        },
        Action {
            action: v!["states:StartExecution", "states:StartExecutionSync"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("ApiState"),
        },
        Action {
            action: v!["events:PutEvents"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("ApiEvents"),
        },
        Action {
            action: v!["sqs:SendMessage", "sqs:GetQueueUrl"],
            effect: s!("Allow"),
            resource: v!["arn:aws:sqs:{{region}}:{{account}}:*"],
            sid: make_sid("ApiQueue"),
        },
    ]
}

fn make_event_actions() -> Vec<Action> {
    vec![
        Action {
            action: v![
                "events:PutTargets",
                "events:PutRule",
                "events:DescribeRule",
                "events:PutEvents"
            ],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("EventEvent"),
        },
        Action {
            action: v!["lambda:InvokeFunction"],
            effect: s!("Allow"),
            resource: v![
                "arn:aws:lambda:{{region}}:{{account}}:function:*",
                "arn:aws:lambda:{{region}}:{{account}}:function:*:*"
            ],
            sid: make_sid("EventLambda"),
        },
        Action {
            action: v!["states:StartExecution"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("EventState"),
        },
        Action {
            action: v!["events:InvokeApiDestination"],
            effect: s!("Allow"),
            resource: v!["*"],
            sid: make_sid("EventApiDest"),
        },
        Action {
            action: v!["appsync:GraphQL"],
            effect: s!("Allow"),
            resource: v!["arn:aws:appsync:{{region}}:{{account}}:apis/*/types/Mutation/fields/*"],
            sid: make_sid("EventMutation"),
        },
    ]
}

fn make_appsync_actions() -> Vec<Action> {
    vec![
        Action {
            action: v!["lambda:InvokeFunction"],
            effect: s!("Allow"),
            resource: v![
                "arn:aws:lambda:{{region}}:{{account}}:function:*",
                "arn:aws:lambda:{{region}}:{{account}}:function:*:*"
            ],
            sid: make_sid("MutationFunction"),
        },
        Action {
            action: v!["appsync:GraphQL"],
            effect: s!("Allow"),
            resource: v!["arn:aws:appsync:{{region}}:{{account}}:apis/*/types/Mutation/fields/*"],
            sid: make_sid("MutationMutation"),
        },
    ]
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Policy {
    #[serde(rename(serialize = "Version", deserialize = "Version"))]
    version: String,
    #[serde(rename(serialize = "Statement", deserialize = "Statement"))]
    statement: Vec<Action>,
}

impl Policy {
    pub fn new(entity: Entity) -> Policy {
        let actions = match entity {
            Entity::Function => make_lambda_actions(),
            Entity::State => make_sfn_actions(),
            Entity::Route => make_api_actions(),
            Entity::Event => make_event_actions(),
            Entity::Mutation => make_appsync_actions(),
            _ => todo!(),
        };

        Policy {
            version: s!("2012-10-17"),
            statement: actions,
        }
    }

    pub fn microvm() -> Policy {
        let actions = make_microvm_actions();

        Policy {
            version: s!("2012-10-17"),
            statement: actions,
        }
    }

    pub fn augment(&self) -> Policy {
        let common = Action {
            action: v!["s3:GetObject", "s3:GetObjectVersion"],
            effect: s!("Allow"),
            resource: v![&format!("arn:aws:s3:::{{{{ASSET_BUCKET}}}}/*")],
            sid: make_sid("LambdaAssetAccess"),
        };

        let mut actions: Vec<Action> = self.statement.clone();
        actions.push(common);
        Policy {
            version: self.version.clone(),
            statement: actions,
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actions_of(policy: &Policy) -> Vec<String> {
        let mut xs: Vec<String> = vec![];
        for st in &policy.statement {
            xs.extend(st.action.clone());
        }
        xs
    }

    /// A route targeting a queue is an SQS-SendMessage integration signed with the
    /// api base role - without this action every queued request 403s at runtime.
    #[test]
    fn api_role_can_send_to_queues() {
        let policy = Policy::new(Entity::Route);
        assert!(actions_of(&policy).contains(&s!("sqs:SendMessage")));
    }

    #[test]
    fn api_role_keeps_its_other_targets() {
        let actions = actions_of(&Policy::new(Entity::Route));
        assert!(actions.contains(&s!("lambda:InvokeFunction")));
        assert!(actions.contains(&s!("states:StartExecution")));
        assert!(actions.contains(&s!("events:PutEvents")));
    }
}
