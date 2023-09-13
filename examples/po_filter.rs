use shop::Loader;
use shop::PurchaseOrderStatus;
use shop::PurchaseOrders;

pub fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: po <item name>");
        std::process::exit(1);
    });

    let item_name = arg;

    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv").unwrap();

    println!();

    let po = purchase_orders
        .filter_by_item_name(item_name)
        .filter_by_status(PurchaseOrderStatus::Billed);
    println!("{:?}", po);

    // let todays_quantity = counted_quantity + restocked_quantity - sold_quantity;
}
