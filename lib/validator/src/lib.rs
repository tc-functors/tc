use colored::Colorize;
use compiler::Entity;
use composer::Topology;

fn validate_gql(topology: &Topology) {
    let types = topology
        .mutations
        .values()
        .into_iter()
        .nth(0)
        .unwrap()
        .types
        .clone();
    let mut graphql: String = "".to_string();
    for (_, v) in types {
        graphql.push_str(&v);
    }

    let diagnostics = graphql_schema_validation::validate(&graphql);
    let formatted_diagnostics = diagnostics
        .iter()
        .map(|err| format!("{}", err))
        .collect::<Vec<String>>();
    for v in formatted_diagnostics {
        if !v.contains("AWS") {
            println!("{}", &v.red());
        }
    }
}

pub async fn validate(topology: &Topology, entity: Entity) {
    match entity {
        Entity::Mutation => validate_gql(&topology),
        _ => println!("nothing"),
    }
}
