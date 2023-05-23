use chrono::NaiveDate;
use shop::Error;
use shop::Invoices;
use shop::Loader;

fn main() -> Result<(), Error> {
    let invoices = Invoices::load("assets/zoho/Invoice.csv").map_err(|source| Error::Load {
        filename: "assets/zoho/Item.csv",
        source,
    })?;
    let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let items = invoices.between(start, end).unique();

    let filename = "examples/output/2023.csv";

    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer.write_record(&["Date", "Name", "Quantity"]).unwrap();

    for item in items.iter() {
        writer.write_record(&["01/05/2023", &item.name(), "0"]).unwrap();
    }
    writer.flush().unwrap();

    Ok(())
}
