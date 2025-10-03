pub fn sandbox() -> String {
    format!("{{{{sandbox}}}}")
}

pub fn profile() -> String {
    format!("{{{{profile}}}}")
}

pub fn account() -> String {
    format!("{{{{account}}}}")
}

pub fn topology_fqn(namespace: &str) -> String {
    format!("{}_{{{{sandbox}}}}", namespace)
}

pub fn maybe_namespace(s: &str) -> String {
    if s.contains("{{sandbox}}") {
        s.to_string()
    } else {
        format!("{{{{namespace}}}}_{}_{{{{sandbox}}}}", s)
    }
}
