use serde::{Deserialize, Serialize};

use crate::records::Records;
use crate::result::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "Item Name")]
    name: String,
    #[serde(rename = "Rate")]
    price: String,
    #[serde(rename = "Purchase Rate")]
    cost: String,
    #[serde(rename = "Stock On Hand", deserialize_with = "parse_quantity")]
    quantity: isize,
}

fn parse_quantity<'de, D>(deserializer: D) -> std::result::Result<isize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(0)
    } else {
        Ok(s.parse::<isize>().unwrap())
    }
}


impl Item {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn price(&self) -> &str {
        &self.price
    }

    pub fn cost(&self) -> &str {
        &self.cost
    }

    pub fn quantity(&self) -> isize {
        self.quantity
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn set_price(&mut self, price: &str) -> &mut Self {
        self.price = price.to_string();
        self
    }

    pub fn set_cost(&mut self, cost: &str) -> &mut Self {
        self.cost = cost.to_string();
        self
    }

    pub fn set_quantity(&mut self, quantity: isize) -> &mut Self {
        self.quantity = quantity;
        self
    }
}

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Records<Item> for Items {}
impl From<Vec<Item>> for Items {
    fn from(vec: Vec<Item>) -> Items {
        Items(vec)
    }
}

impl Items {
    pub fn find_all(&self, name: &str) -> Result<Vec<&Item>> {
        let mut matches = Vec::new();
        for item in &self.0 {
            if item.name.contains(name) {
                matches.push(item);
            }
        }
        Ok(matches)
    }

    pub fn get(&self, item_name: &str) -> Result<&Item> {
        for item in &self.0 {
            if item.name == item_name {
                return Ok(item);
            }
        }
        panic!("item not found: {}", item_name);
    }

    pub fn get_mut(&mut self, item_name: &str) -> Result<&mut Item> {
        for item in &mut self.0 {
            if item.name == item_name {
                return Ok(item);
            }
        }
        panic!("item not found: {}", item_name);
    }
}

