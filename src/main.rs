use std::io;

use slv;

fn main() {
    println!("Welcome to the - Stand Land Vollpfosten - helper!");
    println!();

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
