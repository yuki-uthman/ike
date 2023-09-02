use colored::Colorize;
use shop::Inventories;
use shop::Invoices;
use shop::Items;
use shop::Loader;
use shop::PurchaseOrderStatus;
use shop::PurchaseOrders;

fn update_from_inventory(items: &mut Items, pattern: Option<String>) {
    let inventories = Inventories::load_from_file("assets/Inventory.csv").unwrap();

    println!();
    for item in items.iter_mut() {

        if item.is_counted() {
            item.set_quantity(item.stock_on_hand());
            println!(
                "   ✅ {}: {}pcs\n",
                item.name().blue().bold().strikethrough(),
                item.quantity()
            );
            continue;
        }

        let found = inventories.get(item.name());
        let inventory = match found {
            Ok(inventory) => inventory,
            Err(_) => {
                println!("🧨 {} not in Inventory.csv\n", item.name().red().bold());
                continue;
            }
        };

        let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();
        let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv").unwrap();

        let today = chrono::Local::now().date_naive();

        let counted_date = inventory.date();
        let counted_quantity = inventory.quantity();
        let sold_quantity = invoices
            .between(counted_date, today)
            .count_quantity_sold(inventory.name());

        let restocked_quantity: usize = purchase_orders
            .between(counted_date, today)
            .filter_by_item_name(inventory.name())
            .filter_by_status(PurchaseOrderStatus::Billed)
            .into_quantity()
            .sum();

        let todays_quantity =
            counted_quantity + restocked_quantity as isize - sold_quantity as isize;

        println!("   🔖 {}", inventory.name().green().bold());
        let sold_on_counting_day = invoices
            .on(counted_date)
            .count_quantity_sold(inventory.name());
        if sold_on_counting_day > 0 {
            println!(
                "      {}: {} {}",
                inventory.date().to_string().red().bold(),
                sold_on_counting_day.to_string().red().bold(),
                "pcs sold on the day it was counted!".red().bold(),
            );
        }
        println!(
            "      {} + {} - {} = {}\n",
            counted_quantity.to_string().green(), restocked_quantity.to_string().green(), sold_quantity.to_string().red(), todays_quantity.to_string().green().bold()
        );


        item.set_quantity(todays_quantity);
    }
}

pub fn main() {
    let mut items = Items::load_from_file("assets/Item.csv")
        .unwrap()
        .get_active_items();

    let pattern = std::env::args().nth(1);
    if let Some(pattern) = &pattern {
        println!();
        println!(" 🔍 {} \"{}\"", "Searching for".yellow(), pattern.bright_yellow().bold());
        items = items.find_all(pattern).unwrap();
    }

    items.sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));

    update_from_inventory(&mut items, pattern);

    // let items = items.find_all("sponge").unwrap();
    // items.export("examples/counted/Item.csv").unwrap();
}
