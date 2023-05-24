use colored::Colorize;
use shop::Inventories;
use shop::Invoices;
use shop::Loader;

pub fn main() {
    let mut invoices = Invoices::load("assets/zoho/Invoice.csv").unwrap();
    let inventories = Inventories::load("assets/revision/Inventory.csv").unwrap();

    for inventory in inventories.iter() {
        invoices.set_date(inventory.date());
        let count = invoices.count_frequency(&inventory.name());

        println!("{}", inventory.name().green().bold());
        let today = chrono::Local::now();
        println!(
            "{} ~ {}: {} {}\n",
            inventory.date().to_string(),
            today.date_naive().to_string(),
            count.to_string().green().bold(),
            "times ordered".green()
        );
    }
}
