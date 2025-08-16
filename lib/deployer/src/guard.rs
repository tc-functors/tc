use question::{
    Answer,
    Question,
};

pub fn should_abort(sandbox: &str) -> bool {
    let yes = match std::env::var("CIRCLECI") {
        Ok(_) => false,
        Err(_) => match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => true,
        },
    };
    yes && (sandbox == "stable")
}

pub fn prevent_stable_updates(sandbox: &str) {
    if should_abort(sandbox) {
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
