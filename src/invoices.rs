use serde::{Deserialize, Serialize};

use crate::records::Records;
use crate::result::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Invoice {
    #[serde(rename = "Invoice Date")]
    date: String,
    #[serde(rename = "Invoice Status")]
    status: String,
    #[serde(rename = "Item Name")]
    product: String,
    #[serde(rename = "Quantity")]
    quantity: i32,
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
