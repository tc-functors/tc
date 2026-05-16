use question::{
    Answer,
    Question,
};

use composer::Topology;
use provider::Auth;

use super::aws::{
    event,
    function,
    mutation,
    route,
    state,
};

use compiler::TopologyKind;

pub async fn is_frozen(auth: &Auth, topology: &Topology) -> bool {
    let env = &auth.name;
    match std::env::var("TC_FREEZE") {
        Ok(e) => *env == e,
        Err(_) => {
            let Topology { fqn, kind, ..} = topology;
            match kind {
                TopologyKind::StepFunction => state::is_frozen(auth, fqn).await,
                TopologyKind::Function => function::is_frozen(auth, fqn).await,
                TopologyKind::Graphql => mutation::is_frozen(auth, fqn).await,
                TopologyKind::Routed => route::is_frozen(auth, fqn).await,
                TopologyKind::Evented => event::is_frozen(auth, fqn).await,
            }
        },
    }
}

pub async fn should_abort(auth: &Auth, sandbox: &str, topology: &Topology) -> bool {
    let yes = match std::env::var("CIRCLECI") {
        Ok(_) => is_frozen(auth, topology).await,
        Err(_) => match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => true,
        },
    };
    yes && (sandbox == "stable")
}

pub async fn prevent_stable_updates(auth: &Auth, sandbox: &str, topology: &Topology) {
    if should_abort(auth, sandbox, topology).await {
        std::panic::set_hook(Box::new(|_| {
            println!("Cannot create stable sandbox outside CI");
        }));
        panic!("Cannot create stable sandbox outside CI")
    }
}

pub fn prompt(question: &str) -> bool {
    let answer = Question::new(question)
        .accept("y")
        .accept("n")
        .until_acceptable()
        .show_defaults()
        .confirm();
    answer == Answer::YES
}
