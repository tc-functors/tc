use kit as u;
use colored::Colorize;

fn install_deps(lang: &str) {
    match lang {
        "ruby2.7" => {
            let out = u::sh(
                "gem install rubocop -v 1.37.1 && gem install rubocop-rspec -v 2.4.0",
                &u::pwd(),
            );
            println!("{out}");
        },
        "python3.10" | "python3.11" {

        },
        _ => println!(""),
    }
}
