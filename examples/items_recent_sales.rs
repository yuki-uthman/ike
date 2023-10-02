#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashSet;
use std::ops::Add;

use colored::Colorize;
use serde::{Deserialize, Serialize};
use shop::Invoices;
use shop::Loader;

#[derive(Debug, Clone, PartialEq, Eq, Ord, Deserialize, Serialize)]
struct Frequency {
    item: String,
    count: usize,
}

impl PartialOrd for Frequency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.count == other.count {
            return self.item.partial_cmp(&other.item);
        }
        self.count.partial_cmp(&other.count)
    }
}

impl From<Frequency> for HashSet<String> {
    fn from(frequency: Frequency) -> Self {
        let mut set = HashSet::new();
        set.insert(frequency.item.to_string());
        set
    }
}

impl Add for Frequency {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let count = self.count + other.count;
        let item = self.item + &other.item;
        Self { item, count }
    }
}

fn get_frequency(
    invoices: &Invoices,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> Vec<Frequency> {
    let mut items = invoices.between(start, end).get_sold().unique_items();

    items.sort_by_name();

    let mut frequencies: Vec<Frequency> = Vec::new();
    for item in items.iter() {
        let count = invoices.between(start, end).count_frequency(item.id());

        let frequency = Frequency {
            item: item.name().to_string(),
            count,
        };

        frequencies.push(frequency);
    }
    frequencies.sort();
    frequencies.reverse();
    frequencies
}

fn print_frequencies(frequencies: &[Frequency], minimum: usize) {
    for frequency in frequencies {
        if frequency.count < minimum {
            return;
        }

        println!(
            "{:>3}: {}",
            frequency.count.to_string().green().bold(),
            frequency.item.green(),
        );
    }
}

pub fn main() {
    let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();

    let today = chrono::Local::now().date_naive();
    let september = chrono::NaiveDate::from_ymd_opt(2023, 9, 1).unwrap();
    let august = chrono::NaiveDate::from_ymd_opt(2023, 8, 1).unwrap();
    let july = chrono::NaiveDate::from_ymd_opt(2023, 7, 1).unwrap();
    let june = chrono::NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let may = chrono::NaiveDate::from_ymd_opt(2023, 5, 1).unwrap();
    let april = chrono::NaiveDate::from_ymd_opt(2023, 4, 1).unwrap();
    let march = chrono::NaiveDate::from_ymd_opt(2023, 3, 1).unwrap();
    let february = chrono::NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
    let january = chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

    let mut items = invoices.between(january, today).get_sold().unique_items();

    items.sort_by_name();

    let frequency1 = get_frequency(&invoices, september, today);
    let frequency2 = get_frequency(&invoices, august, september);
    let frequency3 = get_frequency(&invoices, july, august);
    let frequency4 = get_frequency(&invoices, june, july);
    let frequency5 = get_frequency(&invoices, may, june);
    let frequency6 = get_frequency(&invoices, april, may);
    let frequency7 = get_frequency(&invoices, march, april);
    let frequency8 = get_frequency(&invoices, february, march);
    let frequency9 = get_frequency(&invoices, january, february);

    // September
    println!("September");
    print_frequencies(&frequency1, 5);
    println!();

    // August
    println!("August");
    print_frequencies(&frequency2, 5);
    println!();

    println!("July");
    print_frequencies(&frequency3, 5);
    println!();

    println!("June");
    print_frequencies(&frequency4, 5);
    println!();

    println!("May");
    print_frequencies(&frequency5, 5);
    println!();

    println!("April");
    print_frequencies(&frequency6, 5);
    println!();

    println!("March");
    print_frequencies(&frequency7, 5);
    println!();

    println!("February");
    print_frequencies(&frequency8, 5);
    println!();
}
