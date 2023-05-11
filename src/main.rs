use csv;
use serde::{Deserialize, Serialize};

use std::io;

#[derive(Debug, Deserialize, Serialize)]
struct Item {
    #[serde(rename = "Item Name")]
    name: String,
    #[serde(rename = "Rate")]
    price: String,
    #[serde(rename = "Purchase Rate")]
    cost: String,
}

fn main() {
    let mut reader = csv::Reader::from_reader(io::stdin());
    for result in reader.deserialize() {
        let record: Item = result.expect("a CSV record");
        println!("{:?}", record);
    }
}
