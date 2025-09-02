use inquire::{
    InquireError,
    Select,
    Text,
};

pub fn scaffold() {
    let _name = Text::new("Topology name:").prompt();

    let kinds: Vec<&str> = vec!["Evented", "Stepfunctions", "Graphql Mutations"];

    let _kind: Result<&str, InquireError> = Select::new("Select Topology Orchestrator", kinds)
        .without_help_message()
        .prompt();
}
