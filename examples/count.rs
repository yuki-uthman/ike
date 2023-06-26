use colored::Colorize;
use shop::Inventories;
use shop::Invoices;
use shop::Items;
use shop::Loader;

pub fn main() {
    let mut items = Items::load_from_file("assets/Item.csv").unwrap();
    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();
    let inventories = Inventories::load_from_file("assets/Inventory.csv").unwrap();

    for inventory in inventories.iter() {
        let today = chrono::Local::now().date_naive();

        let counted_date = inventory.date();
        let counted_quantity = inventory.quantity();
        let sold_quantity = invoices
            .between(counted_date, today)
            .count_quantity_sold(inventory.name());

        // was there any sales on the day it was counted
        if invoices
            .on(counted_date)
            .count_quantity_sold(inventory.name())
            > 0
        {
            println!(
                "{}: {} pcs sold!",
                inventory.name().green().bold(),
                inventory.quantity().to_string().red().bold()
            );
        }

        let todays_quantity = counted_quantity - sold_quantity;

        items
            .get_mut(inventory.name())
            .unwrap()
            .set_quantity(todays_quantity);
    }

    items.export("examples/counted/Item.csv").unwrap();
}
