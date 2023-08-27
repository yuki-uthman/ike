use colored::Colorize;
use shop::Inventories;
use shop::Invoices;
use shop::Items;
use shop::Loader;
use shop::PurchaseOrderStatus;
use shop::PurchaseOrders;

pub fn main() {
    let mut items = Items::load_from_file("assets/Item.csv").unwrap();
    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();
    let inventories = Inventories::load_from_file("assets/Inventory.csv").unwrap();
    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv").unwrap();

    println!();
    for inventory in inventories.iter() {
        let item = items.get_mut(inventory.name()).unwrap();

        if item.is_counted() {
            let current_quantity = item.stock_on_hand();
            item.set_quantity(current_quantity);

            println!("✅ {}: {}pcs\n", item.name().blue().bold().strikethrough(), item.quantity());
            continue;
        }

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

        let todays_quantity = counted_quantity + restocked_quantity as isize - sold_quantity as isize;

        println!("{}", inventory.name().green().bold());
        println!(
            "   {} + {} - {} = {}",
            counted_quantity, restocked_quantity, sold_quantity, todays_quantity
        );
        println!();

        let sold_on_counting_day = invoices
            .on(counted_date)
            .count_quantity_sold(inventory.name());
        if sold_on_counting_day > 0 {
            println!(
                "   {}: {} {}\n",
                inventory.date().to_string().green().bold(),
                sold_on_counting_day.to_string().red().bold(),
                "pcs sold on the day it was counted!".red().bold(),
            );
        }

        item.set_quantity(todays_quantity);

    }

    let items = items.find_all("round container|alu").unwrap();
//     for item in items.iter() {
//         println!("{}: {}pcs", item.name(), item.quantity());
//     }

    items.export("examples/counted/Item.csv").unwrap();
}
