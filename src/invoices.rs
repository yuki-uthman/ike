use chrono::NaiveDate as Date;
use serde::Deserialize;

use crate::records::Records;
use crate::result::Result;

#[derive(Debug, Deserialize)]
pub struct Invoice {
    #[serde(rename = "Invoice Date", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "Invoice Status")]
    status: String,
    #[serde(rename = "Item Name")]
    product: String,
    #[serde(rename = "Quantity")]
    quantity: i32,
}

fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer).unwrap();
    Ok(Date::parse_from_str(&s, "%Y-%m-%d").unwrap())
}

#[derive(Debug)]
pub struct Invoices(Vec<Invoice>);

impl Records<Invoice> for Invoices {}
impl From<Vec<Invoice>> for Invoices {
    fn from(vec: Vec<Invoice>) -> Invoices {
        Invoices(vec)
    }
}

impl Invoices {
    pub fn new(filename: &'static str) -> Result<Self> {
        Ok(Invoices::load(filename)?)
    }
}
