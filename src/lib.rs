use std::io;

mod cards;
mod dice;
mod timer;

pub fn start_game() {
    println!("Please write down the current timeout:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let timeout = timer::TimeoutFromString::new(input);

    let letter = dice::roll_dice();
    println!("Your chosen letter is: {letter}");
    let card = cards::draw_card();
    println!("Your card contains the following categories:");
    for category in card {
        println!("- {category}")
    }
    timer::start_timer(timeout);
    println!("Your time is over!")
}
