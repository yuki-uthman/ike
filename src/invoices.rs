use serde::{Deserialize, Serialize};

use crate::records::Loader;
use crate::result::Result;

#[derive(Debug)]
pub struct Invoices(Vec<Invoice>);

impl Loader<Invoice> for Invoices {}

impl Invoices {
    pub fn new(filename: &'static str) -> Result<Invoices> {
        let vec = Invoices::load(filename)?;
        Ok(Self(vec))
    }
}

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
