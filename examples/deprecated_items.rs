use chrono::NaiveDate;
use shop::Error;
use shop::Invoices;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoices = Invoices::load("assets/zoho/Invoice.csv").map_err(|source| Error::Load {
        filename: "assets/zoho/Item.csv",
        source,
    })?;

    let start = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
    let items_2022 = invoices.between(start, end).unique();

    let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let items_2023 = invoices.between(start, end).unique();

    let deprecated = items_2022
        .iter()
        .filter(|item| !items_2023.contains(item))
        .collect::<Vec<_>>();

    let filename = "examples/output/deprecated_items.csv";

    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename)?;
    writer.write_record(&["Date", "Name", "Quantity"])?;

    for item in deprecated.iter() {
        writer.write_record(&["01/05/2023", &item.name(), "0"])?;
    }
    writer.flush()?;

    Ok(())
}
