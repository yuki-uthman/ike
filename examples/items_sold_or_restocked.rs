use chrono::NaiveDate;
use shop::Error;
use shop::Invoices;
use shop::Loader;
use shop::PurchaseOrders;

fn main() -> Result<(), Error> {
    let invoices =
        Invoices::load_from_file("assets/Invoice.csv").map_err(|source| Error::Load { source })?;
    let start = NaiveDate::from_ymd_opt(2023, 6, 20).unwrap();
    let today = chrono::Local::now().date_naive();
    let mut sold_items = invoices.between(start, today).unique_items();

    // sort by lowercase name
    sold_items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv")
        .map_err(|source| Error::Load { source })?;
    let start = NaiveDate::from_ymd_opt(2023, 6, 20).unwrap();
    let today = chrono::Local::now().date_naive();
    let mut restocked_items = purchase_orders.between(start, today).unique_items();

    // sort by lowercase name
    restocked_items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    let unique = sold_items + restocked_items;

    let filename = "examples/output/items_to_count.csv";

    std::fs::File::create(filename).unwrap();

    let mut writer = csv::Writer::from_path(filename).unwrap();
    writer.write_record(&["name"]).unwrap();

    for item in unique.iter() {
        if item.name().is_empty() {
            continue;
        }
        writer.write_record(&[item.name().to_string()]).unwrap();
    }
    writer.flush().unwrap();

    Ok(())
}
