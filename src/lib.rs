use std::io;
use std::io::prelude::*;

mod cards;
mod dice;
mod timer;

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

pub fn start_game() {
    println!("Please write down the current timeout:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let timeout = timer::TimeoutFromString::new(input);

    let letter = dice::roll_dice();
    println!("Your letter is: {letter}");
    wait_for_user();

    let card = cards::draw_card(6);
    println!("Your card contains the following categories:");
    for category in card {
        println!("- {category}")
    }
    timer::start_timer(timeout);
    println!("Your time is over!")
}
