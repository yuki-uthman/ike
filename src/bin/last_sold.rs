use colored::Colorize;
use shop::Invoices;
use shop::Loader;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let item_name = &args[1];

    let invoices = Invoices::load("assets/Invoice.csv").unwrap();

    let result = invoices.last_sold(item_name);
    match result {
        Ok(date) => println!("{}: {}", item_name, date.to_string().green().bold()),
        Err(error) => println!("{}: {}", item_name, error.to_string().red().bold()),
    }
}
