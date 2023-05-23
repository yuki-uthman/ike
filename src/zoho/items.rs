use serde::{Deserialize, Serialize};
use std::ops::{Add, Deref, Sub};

use crate::loader::Loader;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("item not found: {name}")]
    ItemNotFound { name: String },

    #[error("no items with the keyword: {keyword}")]
    NoSuchItems { keyword: String },

    #[error("{source}")]
    FileCreate {
        filename: &'static str,
        source: std::io::Error,
    },

    #[error("{source}")]
    FileOpen {
        filename: &'static str,
        source: csv::Error,
    },

    #[error("{source}")]
    Serialization { source: csv::Error },

    #[error("{source}")]
    Flush { source: std::io::Error },
}
type Result<T> = std::result::Result<T, Error>;

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

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Item {
    pub fn new(name: &str, quantity: usize) -> Self {
        Self {
            status: "active".to_string(),
            name: name.to_string(),
            description: "".to_string(),
            sku: "".to_string(),
            usage_unit: "pcs".to_string(),
            price: 0.0,
            cost: 0.0,
            quantity,
            product_type: "goods".to_string(),
            item_type: "inventory".to_string(),
            account: "Inventory Assets".to_string(),
            purchase_account: "Cost of Goods Sold".to_string(),
            inventory_account: "Inventory Assets".to_string(),
            tax_name: "".to_string(),
            tax_type: "".to_string(),
            tax_percentage: "".to_string(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
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
}

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Loader<Item> for Items {}
impl From<Vec<Item>> for Items {
    fn from(vec: Vec<Item>) -> Items {
        Items(vec)
    }
}

impl Deref for Items {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// # Examples
///
/// ```
/// use shop::{Item, Items};
///
/// let items1 = Items::new(vec![Item::new("item1", 1)]);
/// let items2 = Items::new(vec![Item::new("item2", 1)]);
/// let sum = items1 + items2;
/// assert_eq!(sum.len(), 2);
///
/// // only unique items are added
/// let items = Items::new(vec![Item::new("item1", 1)]);
/// let items_duplicate = Items::new(vec![Item::new("item1", 1)]);
/// let sum = items + items_duplicate;
/// assert_eq!(sum.len(), 1);
///
/// ```
impl Add<Items> for Items {
    type Output = Items;

    fn add(mut self, items: Items) -> Self::Output {
        for item in items.0 {
            if !self.0.contains(&item) {
                self.0.push(item);
            }
        }
        self
    }
}

impl Sub<Items> for Items {
    type Output = Items;

    fn sub(mut self, items: Items) -> Self::Output {
        for item in items.0 {
            if let Some(index) = self.0.iter().position(|i| i.name == item.name) {
                self.0.remove(index);
            }
        }
        self
    }
}

impl Items {
    pub fn new(items: Vec<Item>) -> Self {
        Self(items)
    }

    pub fn find_all(&self, name: &str) -> Result<Self> {
        let mut matches = Vec::new();
        for item in &self.0 {
            if item.name.contains(name) {
                matches.push(item.clone());
            }
        }
        if matches.is_empty() {
            return Err(Error::NoSuchItems {
                keyword: name.to_string(),
            });
        }
        Ok(matches.into())
    }

    pub fn get(&self, item_name: &str) -> Result<&Item> {
        for item in &self.0 {
            if item.name == item_name {
                return Ok(item);
            }
        }
        Err(Error::ItemNotFound {
            name: item_name.to_string(),
        })
    }

    pub fn get_mut(&mut self, item_name: &str) -> Result<&mut Item> {
        for item in &mut self.0 {
            if item.name == item_name {
                return Ok(item);
            }
        }
        Err(Error::ItemNotFound {
            name: item_name.to_string(),
        })
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take(&self, how_many: usize) -> Self {
        let items = self.0.iter().take(how_many).cloned().collect::<Vec<_>>();

        items.into()
    }

    pub fn export(&self, filename: &'static str) -> Result<()> {
        std::fs::File::create(filename).map_err(|source| Error::FileCreate { filename, source })?;

        let mut writer = csv::Writer::from_path(filename)
            .map_err(|source| Error::FileOpen { filename, source })?;
        for item in &self.0 {
            writer
                .serialize(item)
                .map_err(|source| Error::Serialization { source })?;
        }
        writer.flush().map_err(|source| Error::Flush { source })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let items = Items::load("tests/assets/zoho/Item.csv").unwrap();
        assert_yaml_snapshot!(items.len(), @r###"
        ---
        694
        "###);
    }
}
