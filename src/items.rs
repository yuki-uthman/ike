use serde::{Deserialize, Serialize};

use crate::records::Records;
use crate::result::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "Item Name")]
    name: String,
    #[serde(rename = "Rate")]
    price: String,
    #[serde(rename = "Purchase Rate")]
    cost: String,
}

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Records<Item> for Items {}
impl From<Vec<Item>> for Items {
    fn from(vec: Vec<Item>) -> Items {
        Items(vec)
    }
}

impl Items {
    pub fn find_all(&self, name: &str) -> Result<Vec<&Item>> {
        let mut matches = Vec::new();
        for item in &self.0 {
            if item.name.contains(name) {
                matches.push(item);
            }
        }
        Ok(matches)
    }
}

