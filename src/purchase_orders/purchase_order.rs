use chrono::NaiveDate as Date;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PurchaseOrder {
    #[serde(rename = "Purchase Order Status")]
    status: String,
    #[serde(rename = "Purchase Order Date", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "Item Name")]
    item_name: String,
    #[serde(rename = "QuantityOrdered")]
    quantity: usize,
}

fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    Ok(Date::parse_from_str(&string, "%Y-%m-%d").map_err(serde::de::Error::custom)?)
}

impl PurchaseOrder {
    pub fn status(&self) -> String {
        self.status.clone()
    }

    pub fn date(&self) -> Date {
        self.date.clone()
    }

    pub fn item_name(&self) -> String {
        self.item_name.clone()
    }

    pub fn quantity(&self) -> usize {
        self.quantity
    }
}
