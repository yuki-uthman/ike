use std::collections::HashSet;
use std::ops::Add;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use shop::Invoices;
use shop::Item;
use shop::Loader;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
struct Profit {
    item: String,
    profit: f32,
}

impl PartialOrd for Profit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.profit == other.profit {
            return self.item.partial_cmp(&other.item);
        }
        self.profit.partial_cmp(&other.profit)
    }
}

// impl PartialOrd for Profit {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         if self.profit == other.profit {
//             return self.item.partial_cmp(&other.item);
//         }
//         self.profit.partial_cmp(&other.profit)
//     }
// }

fn get_profit(
    invoices: &Invoices,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Vec<Profit> {
    let mut items = invoices.between(start, end).get_sold().unique_items();

    items.sort_by_name();

    let mut profits: Vec<Profit> = Vec::new();
    for item in items.iter() {
        let quantity = invoices.between(start, end).count_quantity_sold(item.id());

        let profit = Profit {
            item: item.name().to_string(),
            profit: item.profit() * quantity as f32,
        };

        if profit.item == "TEAK PVC GLOSSY 0.5MM(960) X 22MM" {
            println!("{:#?}", item);
        }

        profits.push(profit);
    }
    profits.sort_by(|a, b| b.partial_cmp(a).unwrap());
    profits
}

fn print_profit(profits: &[Profit], minimum: f32) {
    for profit in profits {
        // if profit.profit < minimum {
        //     return;
        // }

        let item = profit.item.to_string();
        let profit = profit.profit as i32;

        println!("{:>7}: {}", profit.to_string().green().bold(), item.green(),);
    }
}

pub fn main() {
    let mut invoices = Invoices::load_from_file("assets/Invoice.csv")
        .unwrap()
        .get_sold();

    invoices.inject_items();

    // INV-004260
    let invoice = invoices.get_by_invoice_number("INV-004260");
    if let Some(invoice) = invoice {
        println!("{:#?}", invoice);
        println!("{:#?}", invoice.profit());
    }
}
