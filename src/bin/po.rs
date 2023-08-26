use std::ops::Deref;

use chrono::NaiveDate;
use colored::Colorize;
use shop::Error;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: po <item name>");
        std::process::exit(1);
    });

    let items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .find_all(&arg)?;

    for item in items.iter() {
        println!("   {}", item.name().green().bold());
    }

    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv")
        .map_err(|source| Error::Load { source })?;

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

    println!();
    for purchase in purchases.iter() {
        if purchase.purchase_orders.is_empty() {
            continue;
        }

        println!("   {}", purchase.item.name().green().bold());
        for po in purchase.purchase_orders.iter() {
            println!(
                "      {}: {:>5}",
                po.date().to_string().yellow(),
                po.quantity().to_string().blue()
            );
        }
        // total quantity
        let total_quantity: usize = purchase
            .purchase_orders
            .iter()
            .map(|po| po.quantity())
            .sum();
        println!(
            "      {}:      {:>5}",
            "Total".yellow().bold(),
            total_quantity.to_string().blue().bold()
        );
        println!();
    }

    Ok(())
}
