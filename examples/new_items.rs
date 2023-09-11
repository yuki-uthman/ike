use std::ops::Deref;

use chrono::NaiveDate;
use colored::Colorize;
use shop::Error;
use shop::Invoices;
use shop::InvoiceStatus;
use shop::Item;
use shop::Items;
use shop::Loader;
use shop::PurchaseOrder;
use shop::PurchaseOrderStatus;
use shop::PurchaseOrders;

#[derive(Debug)]
struct Purchase {
    item: Item,
    purchase_orders: Vec<PurchaseOrder>,
}

impl Deref for Purchase {
    type Target = Vec<PurchaseOrder>;

    fn deref(&self) -> &Self::Target {
        &self.purchase_orders
    }
}

fn get_purchases(items: &Items, purchase_orders: &PurchaseOrders) -> Vec<Purchase> {
    let mut purchases = Vec::new();
    for item in items.iter() {
        let mut item_purchase = Purchase {
            item: item.clone(),
            purchase_orders: Vec::new(),
        };

        for po in purchase_orders.iter() {
            if po.item_name() == item.name() {
                item_purchase.purchase_orders.push(po.clone());
            }
        }
        purchases.push(item_purchase);
    }
    purchases
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let invoices = Invoices::load_from_file("assets/Invoice.csv")
        .map_err(|source| Error::Load { source })?;

    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv")
        .map_err(|source| Error::Load { source })?;

    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?;

    items.set_created_date(&purchase_orders, &invoices);

    items = items.created_after(NaiveDate::from_ymd_opt(2023, 4, 1).unwrap());

    // sort by created date
    items.sort_by(|a, b| a.created_date().cmp(&b.created_date()));

    for item in items.iter() {
        println!("{}: {}", item.created_date(), item.name());
    }
    // let purchases = get_purchases(&items, &purchase_orders);

    Ok(())
}

