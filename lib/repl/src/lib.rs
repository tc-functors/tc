use colored::Colorize;
use rustyline::{
    DefaultEditor,
    Result,
    error::ReadlineError,
};
use provider::Auth;
use composer::Topology;
use kit as u;

async fn process_cmd(line: &str, auth: &Auth, rt: &Topology) {
    let parts = line.split(' ').collect::<Vec<&str>>();
    let cmd = parts[0];
    match cmd {
        "tree" => {
            composer::pprint(rt, None, "tree");
        },
        "create" => {
            deployer::create(auth, rt, false).await;
        },
        "update" => {
            deployer::try_update(auth, rt, &None).await;
        },
        "delete" => {
            deployer::try_delete(auth, rt, &None, false).await;
        },
        "list" => {
            let entity = Some(String::from("functions"));
            deployer::try_list(auth, rt, &entity).await;
        },

        _ => (),
    }
}

pub async fn start(auth: &Auth, sandbox: &str) -> Result<()> {
    if sandbox == "stable" {
        panic!("Cannot repl with stable sandbox");
    }
    let topology = composer::compose(&u::pwd(), false);
    let rt = resolver::try_resolve(auth, sandbox, &topology, &None, false, true).await;
    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let prompt = format!(
            "{}@{}.{}> ",
            &topology.namespace.blue(),
            &sandbox.cyan(),
            auth.name.green()
        );
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                process_cmd(&line, auth, &rt).await;
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
