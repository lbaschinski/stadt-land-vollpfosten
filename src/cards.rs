use rand::seq::SliceRandom;
use std::fs::read_to_string;
use std::io;

fn load_cards() -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut default_cards = Vec::new();
    let mut junior_cards = Vec::new();
    let mut adult_cards = Vec::new();
    for line in read_to_string("src/cards/default_edition.txt").unwrap().lines() {
        default_cards.push(line.to_string());
    }
    for line in read_to_string("src/cards/junior_edition.txt").unwrap().lines() {
        junior_cards.push(line.to_string());
    }
    for line in read_to_string("src/cards/adult_edition.txt").unwrap().lines() {
        adult_cards.push(line.to_string());
    }
    (default_cards, junior_cards, adult_cards)
}

pub fn choose_collections() -> Vec<String> {
    let (default_cards, junior_cards, adult_cards) = load_cards();

    println!("Please choose up to three card collections:");
    println!("- default");
    println!("- junior");
    println!("- adult");
    let mut card_collections = Vec::new();
    for _ in 0..3 {
        let mut collection = String::new();
        io::stdin()
            .read_line(&mut collection)
            .expect("Failed to read line");
        if collection.trim() == "default" {
            card_collections.extend(default_cards.clone());
        }
        if collection.trim() == "junior" {
            card_collections.extend(junior_cards.clone());
        }
        if collection.trim() == "adult" {
            card_collections.extend(adult_cards.clone());
        }
        if collection == "\n" {
            break
        }
    }
    if card_collections.len() == 0 {
        panic!("Please choose at least one card collection...");
    }
    card_collections.clone()
}

pub fn draw_card(game_card_collection: &Vec<String>, num: u32) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut card: Vec<String> = Vec::new();
    for _ in 0..num {
        let mut category = game_card_collection.choose(&mut rng).unwrap();
        while card.contains(&category) {
            category = game_card_collection.choose(&mut rng).unwrap();
        }
        card.push(category.clone());
    }
    card
}
