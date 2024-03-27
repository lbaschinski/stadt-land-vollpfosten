use std::io;
use std::io::prelude::*;

mod cards;
mod dice;
mod timer;
pub mod web_app;

/// Wait for user input to continue
fn wait_for_user() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line,
    // so we print without a newline and flush manually.
    write!(stdout, "Press Enter (↵) to continue...").unwrap();
    stdout.flush().unwrap();

    // Read everything and discard
    let _ = stdin.read_line(&mut String::new()).unwrap();
}

/// Start one round of the game
fn start_round(category_collection: &Vec<String>) {
    println!("Please write down the current timeout:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let timeout = timer::TimeoutFromString::new(input);
    println!();

    let letter = dice::roll_dice();
    println!("Your letter is: {letter}");
    wait_for_user();
    println!();

    let card = cards::draw_card(category_collection, 6);
    println!("Your card contains the following categories:");
    for category in card {
        println!("- {category}")
    }
    println!();

    timer::start_timer(timeout);
    println!("Your time is over!");
    println!()
}

/// Add new categories to existing collections.
pub fn add_categories() {
    println!("Please choose a category to add to:");
    println!("- default");
    println!("- junior");
    println!("- adult");
    let mut collection = String::new();
    io::stdin()
        .read_line(&mut collection)
        .expect("Failed to read line");
    println!();
    let filename = if collection.trim() == "default" {
        "src/categories/default_edition.txt"
    }
    else if collection.trim() == "junior" {
        "src/categories/junior_edition.txt"
    }
    else if collection.trim() == "adult" {
        "src/categories/adult_edition.txt"
    }
    else {
        println!("Unknown category: {}", collection.trim());
        return ();
    };

    loop {
        println!("Please input the category you want to add or write 'exit' to return:");
        let mut category = String::new();
        io::stdin()
            .read_line(&mut category)
            .expect("Failed to read line");
        if category.trim() == "exit" {
            println!("Returning...");
            println!();
            break
        }
        println!();
        cards::add_category(filename, category.trim());
    }
}

/// Start the game by choosing category collections
/// and starting a new round.
pub fn start_game() {
    let category_collection = cards::choose_collections();

    loop {
        start_round(&category_collection);
        println!("Do you want to start a new round? (y/n)");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input.trim() != "y" {
            println!("Stopping the game...");
            println!();
            break
        }
        println!()
    }
}
