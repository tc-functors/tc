use colored::Colorize;
use rustyline::{
    DefaultEditor,
    Result,
    error::ReadlineError,
};


#[janetrs::janet_fn(arity(fix(1)))]
fn testing(args: &mut [Janet]) -> Janet {
    use janetrs::JanetType::*;
    //let arg = args.get_matches(0, &[Abstract, Buffer]);
    Janet::nil()
}

async fn process_cmd(client: &JanetClient, line: &str) {
    let out = client.run(line).unwrap();
    println!("{out}");
}

pub async fn start() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    let mut client = JanetClient::init_with_default_env().unwrap();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let prompt = format!("{}> ", "tc".cyan());

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                let _ = process_cmd(&client, &line).await;
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}
