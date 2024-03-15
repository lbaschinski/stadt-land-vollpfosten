use std::io;

use slv;

fn main() {
    println!("Welcome to the - Stand Land Vollpfosten - helper!");
    println!();

    let category_collection = slv::cards::choose_collections();

    loop {
        slv::start_round(&category_collection);
        println!("Do you want to start a new round? (y/n)");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input.trim() != "y" {
            println!("Stopping the game...");
            break
        }
        println!()
    }
}
