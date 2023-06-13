use colored::Colorize;
use shop::Items;
use shop::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let first_pattern = &args[1];
    let second_pattern = &args[2];

    let items = Items::load_from_file("assets/Item.csv")?.get_active_items();

    let first_items = items.find_all(first_pattern)?;
    let second_items = items.find_all(second_pattern)?;

    let difference1 = first_items - second_items;

    println!();
    println!("  {}", first_pattern.yellow().bold());
    for item in difference1.iter() {
        println!("     {}", item.name().green().bold());
    }
    println!();


    let first_items = items.find_all(first_pattern)?;
    let second_items = items.find_all(second_pattern)?;

    let difference2 = second_items - first_items;

    println!("  {}", second_pattern.yellow().bold());
    for item in difference2.iter() {
        println!("     {}", item.name().green().bold());
    }
    println!();


    Ok(())
}
