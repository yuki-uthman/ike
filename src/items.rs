use serde::{Deserialize, Serialize};
use crate::result::Result;

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Items {
    pub fn load(filename: &str) -> Result<Items> {
        let mut reader = csv::Reader::from_path(filename)?;
        let mut items = Vec::new();
        for result in reader.deserialize() {
            let record: Item = result?;
            items.push(record);
        }
        Ok(Items(items))
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
