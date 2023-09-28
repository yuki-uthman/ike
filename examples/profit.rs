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
    let invoices = Invoices::load_from_file("assets/Invoice.csv")
        .unwrap()
        .get_sold();

    let today = chrono::Local::now().date_naive();
    let september = chrono::NaiveDate::from_ymd(2023, 9, 1);
    let august = chrono::NaiveDate::from_ymd(2023, 8, 1);
    let july = chrono::NaiveDate::from_ymd(2023, 7, 1);
    let june = chrono::NaiveDate::from_ymd(2023, 6, 1);
    let may = chrono::NaiveDate::from_ymd(2023, 5, 1);
    let april = chrono::NaiveDate::from_ymd(2023, 4, 1);
    let march = chrono::NaiveDate::from_ymd(2023, 3, 1);
    let february = chrono::NaiveDate::from_ymd(2023, 2, 1);
    let january = chrono::NaiveDate::from_ymd(2023, 1, 1);

    let mut items = invoices.between(january, today).get_sold().unique_items();

    items.sort_by_name();

    let frequency1 = get_profit(&invoices, september, today);
    let frequency2 = get_profit(&invoices, august, september);
    let frequency3 = get_profit(&invoices, july, august);
    let frequency4 = get_profit(&invoices, june, july);
    let frequency5 = get_profit(&invoices, may, june);
    let frequency6 = get_profit(&invoices, april, may);
    let frequency7 = get_profit(&invoices, march, april);
    let frequency8 = get_profit(&invoices, february, march);
    let frequency9 = get_profit(&invoices, january, february);

    // September
    println!("September");
    print_profit(&frequency1, 5.0);
    println!();

    // August
    println!("August");
    print_profit(&frequency2, 5.0);
    println!();

    println!("July");
    print_profit(&frequency3, 5.0);
    println!();

    println!("June");
    print_profit(&frequency4, 5.0);
    println!();

    println!("May");
    print_profit(&frequency5, 5.0);
    println!();

    println!("April");
    print_profit(&frequency6, 5.0);
    println!();

    println!("March");
    print_profit(&frequency7, 5.0);
    println!();

    println!("February");
    print_profit(&frequency8, 5.0);
    println!();
}
