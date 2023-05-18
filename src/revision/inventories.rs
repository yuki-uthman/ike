use chrono::NaiveDate as Date;
use serde::Deserialize;
use std::ops::Deref;

use crate::loader::Loader;

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
    let string = String::deserialize(deserializer)?;
    Ok(Date::parse_from_str(&string, "%d/%m/%Y").map_err(serde::de::Error::custom)?)
}

#[derive(Debug)]
pub struct Inventories(Vec<Inventory>);

impl Loader<Inventory> for Inventories {}
impl From<Vec<Inventory>> for Inventories {
    fn from(vec: Vec<Inventory>) -> Inventories {
        Inventories(vec)
    }
}

// https://stackoverflow.com/questions/68277992/implement-iterator-trait-for-a-struct-containing-an-iterable-field
impl Deref for Inventories {
    type Target = Vec<Inventory>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Inventories {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let inventories = Inventories::load("tests/assets/revision/Inventory.csv").unwrap();
        assert_yaml_snapshot!(inventories.len(), @r###"
        ---
        3
        "###);
    }
}
