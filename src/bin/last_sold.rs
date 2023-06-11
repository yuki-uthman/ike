use chrono::Utc;
use colored::Colorize;
use shop::Invoices;
use shop::Items;
use shop::Loader;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pattern = &args[1];

    let items = Items::load_from_file("assets/Item.csv").unwrap();
    let items = items.find_all(pattern).unwrap();

    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let today = Utc::now().naive_utc().date();
    let mut vec = items
        .iter()
        .map(|item| {
            let option = invoices.last_sold(item.name());
            let num_of_days = if let Some(date) = option {
                (today - date).num_days()
            } else {
                9999
            };
            (num_of_days, item)
        })
        .collect::<Vec<_>>();
    vec.sort_by(|a, b| a.0.cmp(&b.0));

    for item in vec {
        let num_of_days = item.0.to_string();
        if item.0 == 9999 {
            println!(
                "{:>5} {}: {}",
                "~~~".red().bold(),
                "days ago".red().bold(),
                item.1.name().red().bold()
            );
        } else {
            println!(
                "{:>5} {}: {}",
                num_of_days.green().bold(),
                "days ago".green().bold(),
                item.1.name().green().bold()
            );
        }
    }
}
