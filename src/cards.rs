use rand::seq::SliceRandom;
use std::fs::read_to_string;
use std::io;

fn load_categories() -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut default_categories = Vec::new();
    let mut junior_categories = Vec::new();
    let mut adult_categories = Vec::new();
    for line in read_to_string("src/categories/default_edition.txt").unwrap().lines() {
        default_categories.push(line.to_string());
    }
    for line in read_to_string("src/categories/junior_edition.txt").unwrap().lines() {
        junior_categories.push(line.to_string());
    }
    for line in read_to_string("src/categories/adult_edition.txt").unwrap().lines() {
        adult_categories.push(line.to_string());
    }
    (default_categories, junior_categories, adult_categories)
}

pub fn choose_collections() -> Vec<String> {
    let (default_categories, junior_categories, adult_categories) = load_categories();

    println!("Please choose up to three category collections:");
    println!("- default");
    println!("- junior");
    println!("- adult");
    let mut category_collections = Vec::new();
    for _ in 0..3 {
        let mut collection = String::new();
        io::stdin()
            .read_line(&mut collection)
            .expect("Failed to read line");
        if collection.trim() == "default" {
            category_collections.extend(default_categories.clone());
        }
        if collection.trim() == "junior" {
            category_collections.extend(junior_categories.clone());
        }
        if collection.trim() == "adult" {
            category_collections.extend(adult_categories.clone());
        }
        if collection == "\n" {
            break
        }
    }
    if category_collections.len() == 0 {
        panic!("Please choose at least one category collection...");
    }
    category_collections.clone()
}

/// Draw a card with `num` categories from `category_collection`.
pub fn draw_card(category_collection: &Vec<String>, num: u32) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut card: Vec<String> = Vec::new();
    for _ in 0..num {
        let mut category = category_collection.choose(&mut rng).unwrap();
        while card.contains(&category) {
            category = category_collection.choose(&mut rng).unwrap();
        }
        card.push(category.clone());
    }
    card
}
