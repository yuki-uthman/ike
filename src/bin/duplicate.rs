use std::collections::HashSet;

use colored::Colorize;
use shop::Items;
use shop::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let first_pattern = &args[1];
    let second_pattern = &args[2];

    let items = Items::load_from_file("assets/Item.csv")?.only_active_items();

    let first_items: HashSet<String> = items.find_all(first_pattern)?.into();
    let second_items: HashSet<String> = items.find_all(second_pattern)?.into();

    let dup = first_items.intersection(&second_items).collect::<Vec<_>>();
    if dup.is_empty() {
        println!();
        println!("  {}", "No Duplicates!".green().bold());
        println!();
        return Ok(());
    }

    println!();
    for name in dup.iter() {
        println!("     {}", name.red().bold());
    }
    println!();

    Ok(())
}
