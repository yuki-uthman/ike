use colored::Colorize;
use shop::Items;
use shop::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() -> Result<()> {
    let items = Items::load_from_file("assets/Item.csv")?.get_active_items();

    let args: Vec<String> = std::env::args().collect();

    let first_pattern = &args[1];
    let mut main_items = items.find_all(first_pattern)?;

    for arg in args.iter().skip(2) {
        let to_remove = items.find_all(arg)?;
        main_items = main_items - to_remove;
    }

    if main_items.is_empty() {
        println!();
        println!("    {}", "All Sieved!".green().bold());
        println!();
        return Ok(());
    }

    println!();
    for item in main_items.iter() {
        println!("     {}", item.name().red().bold());
    }
    println!();

    Ok(())
}
