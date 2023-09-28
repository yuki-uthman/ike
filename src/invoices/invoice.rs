use chrono::NaiveDate as Date;
use serde::Deserialize;

use crate::Item;

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Draft,
    Closed,
    Overdue,
}

#[allow(unused)] // invoice_number never read but required to look up in zoho
#[derive(Clone, Debug, Deserialize)]
pub struct Invoice {
    #[serde(rename = "Invoice Number")]
    invoice_number: String,
    #[serde(rename = "Invoice Date", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "Invoice Status", deserialize_with = "deserialize_status")]
    status: Status,
    #[serde(rename = "Item Name")]
    item_name: String,
    #[serde(rename = "Item Price")]
    item_price: f32,
    #[serde(rename = "Quantity")]
    quantity: usize,
    #[serde(rename = "Product ID", deserialize_with = "deserialize_product_id")]
    product_id: usize,

    #[serde(rename = "Total")]
    total: f32,
    #[serde(rename = "SubTotal")]
    sub_total: f32,

    #[serde(rename = "Item Tax Amount", deserialize_with = "deserialize_tax_amount")]
    item_tax_amount: f32,


    #[serde(skip_deserializing, skip_serializing)]
    item: Option<Item>,
}

fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    Ok(Date::parse_from_str(&string, "%Y-%m-%d").map_err(serde::de::Error::custom)?)
}

fn deserialize_status<'de, D>(deserializer: D) -> std::result::Result<Status, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let status = String::deserialize(deserializer).unwrap();
    match status.as_str() {
        "Draft" => Ok(Status::Draft),
        "Closed" => Ok(Status::Closed),
        "Overdue" => Ok(Status::Overdue),
        _ => Err(serde::de::Error::custom("invalid status")),
    }
}

fn deserialize_product_id<'de, D>(deserializer: D) -> std::result::Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let product_id = String::deserialize(deserializer).unwrap();
    match product_id.parse::<usize>() {
        Ok(product_id) => Ok(product_id),
        Err(_) => Ok(0),
    }
}

fn deserialize_tax_amount<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let tax_amount = String::deserialize(deserializer).unwrap();
    match tax_amount.parse::<f32>() {
        Ok(tax_amount) => Ok(tax_amount),
        Err(_) => Ok(0.0),
    }
}

impl Invoice {
    pub fn set_item(&mut self, item: Item) {
        self.item = Some(item);
    }

    pub fn invoice_number(&self) -> String {
        self.invoice_number.clone()
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn status(&self) -> Status {
        self.status.clone()
    }

    pub fn item_name(&self) -> String {
        self.item_name.clone()
    }

    pub fn quantity(&self) -> usize {
        self.quantity
    }

    pub fn product_id(&self) -> usize {
        self.product_id
    }

    pub fn item_as_ref(&self) -> Option<&Item> {
        self.item.as_ref()
    }
}
