use colored::Colorize;
use serde::{Deserialize, Serialize};
use shop::Invoices;
use shop::Items;
use shop::Loader;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct Frequency {
    item_name: String,
    count: usize,
}

impl PartialOrd for Frequency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.count == other.count {
            return self.item_name.partial_cmp(&other.item_name);
        }
        self.count.partial_cmp(&other.count)
    }
}

impl Ord for Frequency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.count == other.count {
            return self.item_name.cmp(&other.item_name);
        }
        self.count.cmp(&other.count)
    }
}

pub fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let pattern = &args[1];

    let items = Items::load_from_file("assets/Item.csv")
        .unwrap()
        .get_active_items();
    let items = items.find_all(pattern).unwrap();

    let invoices = Invoices::load_from_file("assets/Invoice.csv")?.get_closed();

    let mut frequencies: Vec<Frequency> = Vec::new();
    for item in items.iter() {
        let start = chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let today = chrono::Local::now().date_naive();
        let count = invoices.between(start, today).count_frequency(item.id());

        let frequency = Frequency {
            item_name: item.name().to_string(),
            count,
        };

        frequencies.push(frequency);
    }
    frequencies.sort();
    frequencies.reverse();

    println!();
    for frequency in frequencies {
        let count = frequency.count.to_string();
        let name = frequency.item_name.to_string();

        if count == "0" {
            println!(
                "{:>5} {}: {}",
                count.red().bold(),
                "times".red().bold(),
                name.red().bold()
            );
        } else {
            println!(
                "{:>5} {}: {}",
                count.green().bold(),
                "times".green().bold(),
                name.green().bold()
            );
        }
    }
    println!();

    Ok(())
}
