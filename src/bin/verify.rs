use colored::Colorize;
use shop::ItemTaxName;
use shop::Items;

use shop::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn main() -> Result<()> {
    let items = Items::load_from_file("assets/Item.csv")?;
    let mut items = items.filter(|item| item.tax_name() == ItemTaxName::None);
    items.sort_by_name();

    println!();
    println!("  {}", "Tax Field Missing!!!".red().bold());
    for item in items.iter() {
        println!("     {}", item.name().green().bold());
    }
    println!();

    Ok(())
}
