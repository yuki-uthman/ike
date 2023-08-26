use chrono::NaiveDate as Date;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Inventory {
    #[serde(rename = "Date", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "Item Name")]
    product: String,
    #[serde(rename = "Quantity")]
    quantity: isize,
}

impl Inventory {
    pub fn date(&self) -> Date {
        self.date
    }

    pub fn name(&self) -> &str {
        &self.product
    }

    pub fn quantity(&self) -> isize {
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
