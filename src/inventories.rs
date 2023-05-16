use chrono::NaiveDate as Date;
use serde::Deserialize;

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

impl Inventories {}