#![allow(unused_imports)]

use colored::Colorize;
use shop::Invoices;
use shop::Loader;

pub fn main() {
    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let item_name = "Round Container R12 370Ml 50*10Pcs";
    let sales = invoices
        .iter()
        .filter(|invoice| invoice.item_name() == item_name)
        .collect::<Vec<_>>();

    let launch = sales.first().unwrap().date();
    let today = chrono::Local::now().date_naive();

    // iterate from launch date to today
    for date in launch.iter_days() {
        // let orders_count = invoices.on(date).count_frequency(item.id());
        // println!(
        //     "{}: {} {}",
        //     date.to_string(),
        //     orders_count.to_string().green().bold(),
        //     "times ordered".green()
        // );

        if date > today {
            break;
        }
    }
}
