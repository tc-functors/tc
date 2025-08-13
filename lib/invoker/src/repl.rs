use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

async fn process_cmd(line: &str) {
    let parts = line.split(' ').collect::<Vec<&str>>();
    let cmd = parts[0];
    match cmd {
        "set" => {
            println!("setting");
        }

        _ => (),
    }
}

pub async fn start(profile: &str, namespace: &str, sandbox: &str) -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let prompt = format!(
            "{}@{}.{}> ",
            &sandbox.cyan(),
            &namespace.blue(),
            profile.green()
        );
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                process_cmd(&line).await;
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
