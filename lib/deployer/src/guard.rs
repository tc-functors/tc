use question::{
    Answer,
    Question,
};

pub fn is_frozen(env: &str) -> bool {
    match std::env::var("TC_FREEZE") {
        Ok(e) => env == &e,
        Err(_) => false
    }
}

pub fn should_abort(env: &str, sandbox: &str) -> bool {
    let yes = match std::env::var("CIRCLECI") {
        Ok(_) => is_frozen(env),
        Err(_) => match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => true,
        },
    };
    yes && (sandbox == "stable")
}

pub fn prevent_stable_updates(env: &str, sandbox: &str) {
    if is_frozen(env) {
        std::panic::set_hook(Box::new(|_| {
            println!("QA is frozen. Aborting sandbox update");
        }));
        panic!("QA is frozen. Aborting sandbox update")
    }
    if should_abort(env, sandbox) {
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
