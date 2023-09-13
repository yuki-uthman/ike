use std::fs::File;

use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_active_items();

    let items = items.find_all("890|83241").unwrap();

    // sort by name
    // items.sort_by(|a, b| a.name().cmp(b.name()));

    let filename = "examples/output/items.csv";
    File::create(&filename).unwrap();
    let mut writer = csv::Writer::from_path(filename).unwrap();

    writer
        .write_record(&["Item Name", "Price", "Cost"])
        .unwrap();

    for item in items.iter() {
        writer
            .write_record(&[item.name(), &item.price().to_string(), &item.cost().to_string()])
            .unwrap();
    }

    writer.flush().unwrap();

    Ok(())
}
