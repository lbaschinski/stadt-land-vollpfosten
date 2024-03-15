use rand::seq::SliceRandom;
use std::fs::read_to_string;

pub fn load_cards() -> Vec<String> {
    let mut default_cards = Vec::new();
    for line in read_to_string("src/cards/default_edition.txt").unwrap().lines() {
        default_cards.push(line.to_string());
    }
    default_cards
}

pub fn draw_card(num: u32) -> Vec<String> {
    let default_cards = load_cards();
    let mut rng = rand::thread_rng();
    let mut card: Vec<String> = Vec::new();
    for _ in 0..num {
        let mut category = default_cards.choose(&mut rng).unwrap();
        while card.contains(&category) {
            category = default_cards.choose(&mut rng).unwrap();
        }
        card.push(category.clone());
    }
    card
}
