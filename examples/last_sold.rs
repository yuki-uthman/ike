use std::fs::File;

use chrono::Utc;
use shop::Invoices;
use shop::Items;
use shop::Loader;

pub fn main() {
    let items = Items::load_from_file("assets/Item.csv").unwrap();

    let items: Items = items
        .clone()
        .into_iter()
        .filter(|item| item.is_active())
        .collect::<Vec<_>>()
        .into();

    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let dir = "examples/output";
    let name = "last_sold";
    let filename = format!("{}/{}.csv", dir, name);
    File::create(&filename).unwrap();
    let mut writer = csv::Writer::from_path(filename).unwrap();

    writer
        .write_record(&["Item Name", "Most recent sale"])
        .unwrap();

    let today = Utc::now().naive_utc().date();
    for item in items.iter() {
        let mut row = vec![item.name()];
        let last_sold = invoices.last_sold(item.name());

        let date = if let Some(last_sold) = last_sold {
            let num_of_days = (today - last_sold).num_days().to_string();
            num_of_days
        } else {
            "9999".to_string()
        };

        row.push(&date);
        writer.write_record(&row).unwrap();
    }
    writer.flush().unwrap();
}
