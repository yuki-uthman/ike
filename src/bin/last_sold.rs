use colored::Colorize;
use shop::Invoices;
use shop::Loader;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();
    let item_name = &args[1];

    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let option = invoices.last_sold(item_name);
    match option {
        Some(date) => println!("{}: {}", item_name, date.to_string().green().bold()),
        None => println!("{}: {}", item_name, "Never sold".red().bold()),
    }
}
