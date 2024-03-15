mod cards;
mod dice;
mod timer;

pub fn start_game() {
    let letter = dice::roll_dice();
    println!("Your chosen letter is: {letter}");
    let card = cards::draw_card();
    println!("Your card contains the following categories:");
    for category in card {
        println!("- {category}")
    }
    timer::start_timer();
    println!("Your time is over!")
}
