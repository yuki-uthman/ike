use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::Tag;
use super::Tags;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Item Name")]
    name: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "SKU")]
    sku: String,
    #[serde(rename = "Usage unit")]
    usage_unit: String,

    #[serde(rename = "Rate", deserialize_with = "trim_currency")]
    price: f32,
    #[serde(rename = "Purchase Rate", deserialize_with = "trim_currency")]
    cost: f32,
    #[serde(
        skip_deserializing,
        default = "reset_quantity",
        rename(serialize = "Initial Stock")
    )]
    quantity: usize,

    #[serde(rename = "Product Type")]
    product_type: String,
    #[serde(rename = "Item Type")]
    item_type: String,

    #[serde(rename = "Account")]
    account: String,
    #[serde(rename = "Purchase Account")]
    purchase_account: String,
    #[serde(rename = "Inventory Account")]
    inventory_account: String,

    #[serde(rename = "Tax Name")]
    tax_name: String,
    #[serde(rename = "Tax Type")]
    tax_type: String,
    #[serde(rename = "Tax Percentage")]
    tax_percentage: String,

    #[serde(rename = "CF.tags", deserialize_with = "de_tags", serialize_with = "se_tags")]
    tags: Tags,
}

fn reset_quantity() -> usize {
    0
}

/// price and cost fields are in the format "MVR 1.00"
/// this function trims the "MVR " prefix and parses the rest as f32
fn trim_currency<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let s = string.trim_start_matches("MVR ");
    Ok(s.parse().map_err(serde::de::Error::custom)?)
}

fn de_tags<'de, D>(deserializer: D) -> std::result::Result<Tags, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let result = String::deserialize(deserializer);
    // if string is not empty, split it by comma and parse each category
    // else return an empty vector
    if result.is_err() {
        return Ok(Tags::new());
    }

    let string = result.unwrap();
    if string.is_empty() {
        return Ok(Tags::new());
    }

    let tags = Tags::from_str(&string).unwrap();
    Ok(tags)
}

fn se_tags<S>(tags: &Tags, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string = String::from(tags);
    serializer.serialize_str(&string)
}


impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Item {
    pub fn new(name: &str) -> Self {
        Self {
            status: "Active".to_string(),
            name: name.to_string(),
            description: "".to_string(),
            sku: "".to_string(),
            usage_unit: "pcs".to_string(),
            price: 0.0,
            cost: 0.0,
            quantity: 0,
            product_type: "goods".to_string(),
            item_type: "inventory".to_string(),
            account: "Inventory Assets".to_string(),
            purchase_account: "Cost of Goods Sold".to_string(),
            inventory_account: "Inventory Assets".to_string(),
            tax_name: "".to_string(),
            tax_type: "".to_string(),
            tax_percentage: "".to_string(),
            tags: Tags::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn price(&self) -> f32 {
        self.price
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn quantity(&self) -> usize {
        self.quantity
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn set_price(&mut self, price: f32) -> &mut Self {
        self.price = price;
        self
    }

    pub fn set_cost(&mut self, cost: f32) -> &mut Self {
        self.cost = cost;
        self
    }

    pub fn set_quantity(&mut self, quantity: usize) -> &mut Self {
        self.quantity = quantity;
        self
    }

    pub fn is_active(&self) -> bool {
        self.status == "Active"
    }

    pub fn tagged(&self, tag: Tag) -> bool {
        self.tags.contains(&tag)
    }
}
