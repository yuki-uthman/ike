use colored::*;
use shop::Invoices;
use shop::Item;
use shop::Items;
use shop::Loader;
use shop::PurchaseOrders;

struct History {
    item: Item,
    bought: usize,
    sold: usize,
}

pub fn main() {
    let item_pattern = std::env::args().nth(1).unwrap_or_else(|| {
        println!("need <item pattern>");
        std::process::exit(1);
    });

    let items = Items::load_from_file("assets/Item.csv")
        .unwrap()
        .get_active_items()
        .get_uncounted_items()
        .find_all(&item_pattern)
        .unwrap();

    let po = PurchaseOrders::load_from_file("assets/Purchase_Order.csv").unwrap();

    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let mut histories = Vec::new();
    for item in items.iter() {
        let bought = po.filter_by_item_id(item.id()).into_quantity().sum();
        let sold = invoices.count_quantity_sold(item.id());

        let history = History {
            item: item.clone(),
            bought,
            sold,
        };

        histories.push(history);
    }

    for history in histories.iter() {
        let calculated_stock = history.bought as isize - history.sold as isize;
        let stock_from_zoho = history.item.stock_on_hand();

        if calculated_stock == stock_from_zoho {
            println!("   {}", history.item.name().green().bold());
            println!("      Bought: {}", history.bought);
            println!("      Sold:   {}", history.sold);
            println!("      Bought - Sold :  {}", calculated_stock);
            println!("      Zoho Stock:  {}", history.item.stock_on_hand());
            println!();
        } else {
            println!("   {}", history.item.name().red().bold());
            println!("      Bought: {}", history.bought);
            println!("      Sold:   {}", history.sold);
            println!("      Bought - Sold :  {}", calculated_stock);
            println!("      Zoho Stock:  {}", history.item.stock_on_hand());
            println!();
        }
    }
}
