use colored::Colorize;
use shop::Invoices;
use shop::Loader;

pub fn main() {

    let item_name = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: invoices <item name>");
        std::process::exit(1);
    });

    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let sales = invoices
        .iter()
        .filter(|invoice| invoice.item_name() == item_name)
        .collect::<Vec<_>>();

    let launch = sales.first().unwrap().date();
    let today = chrono::Local::now().date_naive();

    println!();
    println!(
        "   {}",
        item_name.green().bold()
    );
    // iterate from launch date to today
    for date in launch.iter_days() {
        let orders_count = invoices.on(date).count_frequency(&item_name);
        println!(
            "     {}: {} {}",
            date.to_string(),
            orders_count.to_string().green().bold(),
            "times ordered".green()
        );

        if date > today {
            break;
        }
    }
}
