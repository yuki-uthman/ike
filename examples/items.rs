use std::fs::File;

use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_active_items();

    // filter the items with stock on hand with minus

    // sort by lowercase name
    items.sort_by_name();

    let filename = "examples/output/items.csv";
    File::create(&filename).unwrap();
    let mut writer = csv::Writer::from_path(filename).unwrap();

    writer
        .write_record(&["Item Name", "Price", "Cost"])
        .unwrap();

    for item in items.iter() {
        writer
            .write_record(&[
                item.name(),
                &item.price().to_string(),
                &item.cost().to_string(),
            ])
            .unwrap();
    }

    writer.flush().unwrap();

    Ok(())
}
