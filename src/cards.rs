use rand::seq::SliceRandom;
use std::fs::{OpenOptions, read_to_string};
use std::io::{self, prelude::*};

pub fn load_categories(name: &str) -> Vec<String> {
    let mut categories = Vec::new();
    for line in read_to_string(format!("src/categories/{name}_edition.txt")).unwrap().lines() {
        categories.push(line.to_string());
    }
    categories
}

pub fn add_category(filename: &str, category: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(filename)
        .unwrap();
    if let Err(e) = writeln!(file, "{}", category) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

pub fn choose_collections() -> Vec<String> {
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
            let default_categories = load_categories(collection.trim());
            category_collections.extend(default_categories.clone());
        }
        if collection.trim() == "junior" {
            let junior_categories = load_categories(collection.trim());
            category_collections.extend(junior_categories.clone());
        }
        if collection.trim() == "adult" {
            let adult_categories = load_categories(collection.trim());
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
