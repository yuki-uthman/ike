use std::fs::File;

use colored::Colorize;
use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_active_items()
        .get_counted_items();

    // sort by name lowercase
    items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    for item in items.iter() {
        let quantity = item.stock_on_hand();
        if quantity < 0 {
            println!("   {}: {}", item.name().red().bold(), quantity);
        }
    }

    Ok(())
}
