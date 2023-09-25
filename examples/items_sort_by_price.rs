use shop::Items;
use shop::Loader;
use shop::PurchaseOrderStatus;
use shop::PurchaseOrders;

fn main() {
    let mut items = Items::load_from_file("assets/Item.csv").unwrap().get_active_items();
    // sort by rate of item
    items.sort_by(|a, b| b.price().partial_cmp(&a.price()).unwrap());

    for item in items.iter() {
        println!("{:>8}: {}", item.price(), item.name());
    }

}
