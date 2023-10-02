use colored::Colorize;
use shop::Invoices;
use shop::Items;
use shop::Loader;

pub fn main() {
    let item_name = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: invoices <item name>");
        std::process::exit(1);
    });

    let invoices = Invoices::load_from_file("assets/Invoice.csv")
        .unwrap()
        .get_sold();

    let items = Items::load_from_file("assets/Item.csv").unwrap();
    let matches = items.find_all(&item_name).unwrap();

    for item in matches.iter() {
        let sales = invoices.filter_by_item_id(item.id());

        if sales.len() == 0 {
            continue;
        }

        let launch = sales.first().unwrap().date();
        let today = chrono::Local::now().date_naive();

        println!();
        println!("   {}", item.name().green().bold());

        let mut total = 0;
        for date in launch.iter_days() {
            let orders_count = invoices.on(date).count_quantity_sold(item.id());
            total += orders_count;

            if orders_count == 0 {
                if date > today {
                    break;
                }
                continue;
            }

            println!(
                "     {}: {} {}",
                date,
                orders_count.to_string().green().bold(),
                "pcs sold".green()
            );

            if date > today {
                break;
            }
        }
        println!(
            "     {}      {}",
            "Total:".green().bold(),
            total.to_string().green().bold()
        );
        println!();
    }
}
