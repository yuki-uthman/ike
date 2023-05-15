use serde::{Deserialize, Serialize};

use crate::records::Loader;
use crate::result::Result;

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Loader<Item> for Items {}

impl Items {
    pub fn new(filename: &'static str) -> Result<Items> {
        let items = Items::load(filename)?;
        Ok(Self(items))
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
