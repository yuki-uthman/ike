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
    pub fn new(filename: &'static str) -> Result<Self> {
        Ok(Items::load(filename)?)
    }

    pub fn get(&self, name: &str) -> Option<&Item> {
        self.0.iter().find(|item| item.name == name)
    }
}

