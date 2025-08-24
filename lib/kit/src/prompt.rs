
use question::{
    Answer,
    Question,
};

fn prompt(question: &str) -> bool {
    let answer = Question::new(question)
        .accept("y")
        .accept("n")
        .until_acceptable()
        .show_defaults()
        .confirm();
    answer == Answer::YES
}
