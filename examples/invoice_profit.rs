#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::fs::File;

use chrono::NaiveDate as Date;
use colored::Colorize;
use serde::Deserialize;
use serde::Serialize;
use shop::Invoices;
use shop::Loader;

#[derive(Debug, Clone, PartialEq)]
struct Profit {
    date: Date,
    profit: i32,
}

pub fn main() {
    let mut invoices = Invoices::load_from_file("assets/Invoice.csv")
        .unwrap()
        .get_sold();

    invoices.inject_items();

    invoices.remove_by_item_name("TEAK PVC GLOSSY 0.5MM(960) X 22MM");

    // September
    let start = chrono::NaiveDate::from_ymd_opt(2023, 9, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2023, 9, 28).unwrap();

    let mut profits = Vec::new();
    for date in start.iter_days() {
        let profit = invoices.on(date).count_profit() as i32;

        profits.push(Profit { date, profit });

        println!(
            "{}: {:>8}",
            date.format("%Y-%m-%d"),
            profit.to_string().green()
        );

        if date == end {
            break;
        }
    }

    let filename = "examples/output/profit.csv";
    File::create(&filename).unwrap();
    let mut writer = csv::Writer::from_path(filename).unwrap();

    writer
        .write_record(&["Day", "Profit"])
        .unwrap();

    for profit in profits.iter() {
        writer
            .write_record(&[
                profit.date.format("%d").to_string(),
                profit.profit.to_string(),
            ])
            .unwrap();
    }

    writer.flush().unwrap();
}
