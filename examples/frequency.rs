use serde::{Deserialize, Serialize};
use shop::Invoices;
use shop::Items;
use shop::Loader;

#[derive(Debug, Clone, PartialEq, Eq, Ord, Deserialize, Serialize)]
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

pub fn main() {
    let invoices = Invoices::load("assets/Invoice.csv").unwrap();
    let items = Items::load("assets/Item.csv").unwrap().get_active_items();

    let mut frequencies: Vec<Frequency> = Vec::new();
    for item in items.iter() {
        // 2022-01-01
        let start = chrono::NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end = chrono::Local::now().date_naive();
        let count = invoices.between(start, end).count_frequency(&item.name());

        let frequency = Frequency {
            item_name: item.name().to_string(),
            count,
        };

        frequencies.push(frequency);
    }
    frequencies.sort();

    let filename = "examples/output/frequency.csv";
    let mut writer = csv::Writer::from_path(filename).unwrap();

    for frequency in frequencies {
        writer.serialize(frequency).unwrap();
    }
    writer.flush().unwrap();
}
