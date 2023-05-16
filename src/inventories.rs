use chrono::NaiveDate as Date;
use serde::Deserialize;
use std::ops::Deref;

use crate::records::Records;

#[derive(Debug, Deserialize)]
pub struct Inventory {
    #[serde(rename = "Date", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "Item Name")]
    product: String,
    #[serde(rename = "Quantity")]
    quantity: usize,
}

impl Inventory {
    pub fn date(&self) -> Date {
        self.date
    }

    pub fn name(&self) -> &str {
        &self.product
    }

    pub fn quantity(&self) -> usize {
        self.quantity
    }
}

fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer).unwrap();
    Ok(Date::parse_from_str(&string, "%Y-%m-%d").unwrap())
}

#[derive(Debug)]
pub struct Inventories(Vec<Inventory>);

impl Records<Inventory> for Inventories {}
impl From<Vec<Inventory>> for Inventories {
    fn from(vec: Vec<Inventory>) -> Inventories {
        Inventories(vec)
    }
}

impl Deref for Inventories {
    type Target = Vec<Inventory>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Inventories {}
