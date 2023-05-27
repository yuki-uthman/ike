use chrono::NaiveDate;
use shop::Error;
use shop::Invoices;
use shop::Loader;

fn main() -> Result<(), Error> {
    let invoices = Invoices::load("assets/zoho/Invoice.csv").map_err(|source| Error::Load {
        filename: "assets/zoho/Item.csv",
        source,
    })?;
    let start = NaiveDate::from_ymd_opt(2021, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let items = invoices.between(start, end).unique_items();

    let filename = "examples/output/items.csv";

    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer.write_record(&["id", "name"]).unwrap();

    let mut id = 1;
    for item in items.iter() {
        if item.name().is_empty() {
            continue;
        }
        writer
            .write_record(&[id.to_string(), item.name().to_string()])
            .unwrap();
        id += 1;
    }
    writer.flush().unwrap();

    Ok(())
}
