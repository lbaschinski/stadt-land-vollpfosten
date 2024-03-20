use std::{env, io, process};

use slv;

#[tokio::main]
async fn main() {
    println!("Welcome to the - Stand Land Vollpfosten - helper!");
    println!();

    let mut args = env::args();
    args.next(); // ignore file name
    let modi = match args.next() {
        Some(arg) => arg,
        None => "webapp".to_string(),
    };

    if modi == "webapp" {
        slv::web_app::serve().await;
    }
    else if modi == "terminal" {
        loop {
            println!("Please choose what you want to do:");
            println!("- add (add categories)");
            println!("- play (start playing the game)");
            println!("- exit (stop execution)");
            let mut action = String::new();
            io::stdin()
                .read_line(&mut action)
                .expect("Failed to read line");
            println!();
            if action.trim() == "add" {
                slv::add_categories();
                continue
            }
            if action.trim() == "play" {
                slv::start_game();
                continue
            }
            if action.trim() == "exit" {
                break
            }
        }
    }
    else {
        eprintln!("Unknown input '{}', must be either 'terminal' or 'webapp'.", modi);
        process::exit(1);
    }
}
