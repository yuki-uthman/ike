use colored::Colorize;
use shop::Invoices;
use shop::Loader;

pub fn main() {
    let mut invoices = Invoices::load_from_file("assets/Invoice.csv")
        .unwrap()
        .get_sold();

    invoices.inject_items();

    // September
    let start = chrono::NaiveDate::from_ymd(2023, 9, 1);
    let end = chrono::NaiveDate::from_ymd(2023, 9, 28);

    for day in start.iter_days() {
        let profit = invoices.on(day).count_profit() as i32;
        println!("{}: {:>8}", day.format("%Y-%m-%d"), profit.to_string().green());

        if day == end {
            break;
        }
    }
}
