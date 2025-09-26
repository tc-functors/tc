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
    Some(format!("TcBase{}{{{{sandbox}}}}", ec))
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
            sid: make_sid("LambdaLog"),
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
    pub version: String,
    #[serde(rename(serialize = "Statement", deserialize = "Statement"))]
    pub statement: Vec<Action>,
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

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
