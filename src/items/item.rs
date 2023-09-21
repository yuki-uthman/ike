use super::Tag;
use super::Tags;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use chrono::NaiveDate as Date;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum TaxName {
    GST,
    None,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(skip_deserializing, default = "reset_date")]
    created_date: Date,

    #[serde(rename = "Item ID")]
    id: usize,
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
    quantity: isize,

    #[serde(rename = "Stock On Hand", deserialize_with = "de_stock_on_hand")]
    stock_on_hand: isize,

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

    #[serde(rename = "Tax Name", deserialize_with = "deserialize_tax_name")]
    tax_name: TaxName,
    #[serde(rename = "Tax Type")]
    tax_type: String,
    #[serde(rename = "Tax Percentage")]
    tax_percentage: String,

    #[serde(
        rename = "CF.tags",
        deserialize_with = "de_tags",
        serialize_with = "se_tags"
    )]
    tags: Tags,
    #[serde(rename = "CF.group")]
    group: String,
}

fn reset_quantity() -> isize {
    0
}

fn reset_date() -> Date {
    Date::from_ymd_opt(2022, 1, 1).unwrap()
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

fn deserialize_tax_name<'de, D>(deserializer: D) -> std::result::Result<TaxName, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let status = String::deserialize(deserializer).unwrap();
    match status.as_str() {
        "GST" => Ok(TaxName::GST),
        "" => Ok(TaxName::None),
        _ => {
            let err = format!("invalid tax name: {}", status);
            Err(serde::de::Error::custom(err))
        }
    }
}

fn de_stock_on_hand<'de, D>(deserializer: D) -> std::result::Result<isize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;

    if string.is_empty() {
        Ok(0)
    } else {
        Ok(string.parse().map_err(serde::de::Error::custom)?)
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl AsRef<str> for Item {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Item {
    pub fn new(name: &str) -> Self {
        Self {
            created_date: Date::from_ymd_opt(2022, 1, 1).unwrap(),
            id: 0,
            status: "Active".to_string(),
            name: name.to_string(),
            description: "".to_string(),
            sku: "".to_string(),
            usage_unit: "pcs".to_string(),
            price: 0.0,
            cost: 0.0,
            quantity: 0,
            stock_on_hand: 0,
            product_type: "goods".to_string(),
            item_type: "inventory".to_string(),
            account: "Inventory Assets".to_string(),
            purchase_account: "Cost of Goods Sold".to_string(),
            inventory_account: "Inventory Assets".to_string(),
            tax_name: TaxName::GST,
            tax_type: "".to_string(),
            tax_percentage: "".to_string(),
            tags: Tags::new(),
            group: "".to_string(),
        }
    }

    pub fn created_date(&self) -> Date {
        self.created_date
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn price(&self) -> f32 {
        self.price
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn quantity(&self) -> isize {
        self.quantity
    }

    pub fn stock_on_hand(&self) -> isize {
        self.stock_on_hand
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn tax_name(&self) -> TaxName {
        self.tax_name
    }

    pub fn tags(&self) -> &Tags {
        &self.tags
    }

    pub fn set_created_date(&mut self, date: Date) -> &mut Self {
        self.created_date = date;
        self
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn set_sku(&mut self, sku: &str) -> &mut Self {
        self.sku = sku.to_string();
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

    pub fn set_quantity(&mut self, quantity: isize) -> &mut Self {
        self.quantity = quantity;
        self
    }

    pub fn is_active(&self) -> bool {
        self.status == "Active"
    }

    pub fn tagged(&self, tag: Tag) -> bool {
        self.tags.contains(&tag)
    }

    pub fn add_tag(&mut self, tag: Tag) -> &mut Self {
        self.tags.insert(tag);
        self
    }

    pub fn add_tags(&mut self, tags: &[Tag]) -> &mut Self {
        for tag in tags {
            self.tags.insert(tag.clone());
        }
        self
    }

    pub fn is_counted(&self) -> bool {
        self.tagged(Tag::Counted)
    }
}
