use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Items {
    pub fn new() -> Self {
        load("Item.csv")
    }

    pub fn get(&self, name: &str) -> Option<&Item> {
        self.0.iter().find(|item| item.name == name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "Item Name")]
    name: String,
    #[serde(rename = "Rate")]
    price: String,
    #[serde(rename = "Purchase Rate")]
    cost: String,
}

fn load(filename: &str) -> Items {
    let mut reader = csv::Reader::from_path(filename).expect("Unable to open file");
    let mut items = Vec::new();
    for result in reader.deserialize() {
        let record: Item = result.expect("a CSV record");
        items.push(record);
    }
    Items(items)
}
