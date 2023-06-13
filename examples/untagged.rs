use std::collections::HashSet;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use shop::Item;
use shop::Items;
use shop::Loader;

pub fn main() -> Result<(), Box<dyn std::error::Error>>{
    let items = Items::load_from_file("assets/Item.csv")?.get_active_items();
    let items_len = items.len();

    let tagged = Items::load_from_dir("examples/tagged")?.get_unique_items();
    let tagged_len = tagged.len();

    let mut untagged = items - tagged;
    let untagged_len = untagged.len();

    untagged.sort_by(|a, b| a.name().cmp(b.name()));
    for item in untagged.iter() {
        println!("{}", item.name());
    }
    println!();

    println!("{}   : {}", "Total".bright_green().bold(), items_len.to_string().bright_green().bold());
    println!("{}  : {}", "tagged".green().bold(), tagged_len.to_string().green().bold());
    println!("{}: {}", "untagged".red().bold(), untagged_len.to_string().red().bold());
    untagged.export("examples/output/untagged.csv")?;
    // let tagged_set = HashSet::from(tagged);

    // let untagged = item_set.difference(&tagged_set);
    // for item in untagged {
    //     println!("{}", item.name());
    // }

    // println!("tagged: {}", tagged.len());
    // for item in tagged.iter() {
    //     println!("{}", item.name());
    // }

    // let filename = "examples/output/frequency.csv";
    // let mut writer = csv::Writer::from_path(filename).unwrap();
    // for frequency in frequencies {
    //     writer.serialize(frequency).unwrap();
    // }
    // writer.flush().unwrap();

    Ok(())
}
