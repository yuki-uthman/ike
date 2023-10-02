use colored::Colorize;
use shop::Items;
use shop::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let items = Items::load_from_file("assets/Item.csv")?.get_active_items();

    println!();
    for pattern in args.iter().skip(1) {
        let mut items = items.find_all(pattern)?;
        items.sort_by_name();

        println!("   {}", pattern.yellow().bold());
        for item in items.iter() {
            println!("     {}", item.name().green().bold());
        }
        println!();
    }

    Ok(())
}
