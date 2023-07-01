use chrono::NaiveDate;
use shop::Error;
use shop::Loader;
use shop::PurchaseOrders;
use shop::PurchaseOrderStatus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let po_items = POs::load_from_file("bibu/po.csv").map_err(|source| Error::Load { source })?;

    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv")
        .map_err(|source| Error::Load { source })?;

    let start = NaiveDate::from_ymd_opt(2022, 6, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    let quantity: usize = purchase_orders
        .between(start, end)
        .filter_by_item_name("Aluminum Foil Box 8368 With Lid 680Ml 1 x 900pcs UAE")
        .filter_by_status(PurchaseOrderStatus::Draft)
        .into_quantity()
        .sum();

    println!("{} pcs", quantity);

    // implement count

    // let foil = purchase_orders
    //     .filter(|po| po.item_name() == "Aluminum Foil Box 8368 With Lid 680Ml 1 x 900pcs UAE");

    // for po in foil {
    //     println!("{:?}", po);
    // }

    // let foil =
    //     purchase_orders.filter_by_item("Aluminum Foil Box 8368 With Lid 680Ml 1 x 900pcs UAE");
    // for po in foil {
    //     println!("{:?}", po);
    // }

    //     let items = Items::load_from_file("assets/Item.csv")
    //         .map_err(|source| Error::Load { source })?
    //         .get_active_items();

    //     for po_item in po_items.iter() {
    //         if items.contains(&po_item.name) {
    //         } else {
    //             println!("{}: {}", "New!".yellow().bold(), po_item.name);
    //         }
    //     }

    let number: u8 = 3;
    number.checked_add(3).unwrap();

    Ok(())
}
