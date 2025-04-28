pub fn role_arn(name: &str) -> String {
    format!("arn:aws:iam::{{{{account}}}}:role/{}", name)
}

pub fn policy_arn(name: &str) -> String {
    format!("arn:aws:iam::{{{{account}}}}:policy/{}", name)
}

pub fn event_bus_arn(bus_name: &str) -> String {
    format!(
        "arn:aws:events:{{{{region}}}}:{{{{account}}}}:event-bus/{}",
        bus_name
    )
}

pub fn sandbox() -> String {
    format!("{{{{sandbox}}}}")
}

pub fn profile() -> String {
    format!("{{{{profile}}}}")
}

pub fn account() -> String {
    format!("{{{{account}}}}")
}

pub fn sqs_arn(name: &str) -> String {
    format!("arn:aws:sqs:{{{{region}}}}:{{{{account}}}}:{}", name)
}

pub fn _layer_arn(name: &str) -> String {
    format!(
        "arn:aws:lambda:{{{{region}}}}:{{{{account}}}}:layer:{}",
        name
    )
}

pub fn lambda_arn(name: &str) -> String {
    format!(
        "arn:aws:lambda:{{{{region}}}}:{{{{account}}}}:function:{}",
        name
    )
}

pub fn sfn_arn(name: &str) -> String {
    format!(
        "arn:aws:states:{{{{region}}}}:{{{{account}}}}:stateMachine:{}",
        name
    )
}

pub fn _base_role(name: &str) -> String {
    format!("tc-base-{}-role", name)
}

pub fn _base_policy(name: &str) -> String {
    format!("tc-base-{}-policy", name)
}

pub fn topology_fqn(namespace: &str, hyphenated_names: bool) -> String {
    if hyphenated_names {
        format!("{}-{{{{sandbox}}}}", namespace)
    } else {
        format!("{}_{{{{sandbox}}}}", namespace)
    }
}

pub fn api_integration_arn(name: &str) -> String {
    format!(
            "arn:aws:apigateway:{{{{region}}}}:lambda:path/2015-03-31/functions/{}/invocations",
        lambda_arn(name)
    )
}
