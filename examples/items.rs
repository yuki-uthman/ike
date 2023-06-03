use std::fs::File;

use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = Items::load("assets/Item.csv").map_err(|source| Error::Load { source })?;

    // filter inactive items
    let items: Items = items
        .clone()
        .into_iter()
        .filter(|item| item.is_active())
        .collect::<Vec<_>>()
        .into();

    // sort by name
    let mut items = items.iter().collect::<Vec<_>>();
    items.sort_by(|a, b| a.name().cmp(b.name()));

    let filename = "examples/output/items.csv";
    File::create(&filename).unwrap();
    let mut writer = csv::Writer::from_path(filename).unwrap();

    writer
        .write_record(&["Item Name", "Quantity", "Date"])
        .unwrap();

    for item in items.iter() {
        writer
            .write_record(&[item.name(), "0", "01/06/2023"])
            .unwrap();
    }

    writer.flush().unwrap();

    Ok(())
}
