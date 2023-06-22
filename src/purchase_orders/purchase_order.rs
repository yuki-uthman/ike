use chrono::NaiveDate as Date;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Status {
    Draft,
    Billed,
    Issued,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PurchaseOrder {
    #[serde(rename = "Purchase Order Status", deserialize_with = "deserialize_status")]
    status: Status,
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

fn deserialize_status<'de, D>(deserializer: D) -> std::result::Result<Status, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let status = String::deserialize(deserializer).unwrap();
    match status.as_str() {
        "Draft" => Ok(Status::Draft),
        "Billed" => Ok(Status::Billed),
        "Issued" => Ok(Status::Issued),
        _ => {
            let msg = format!("invalid status: {}", status);
            Err(serde::de::Error::custom(msg))
        }
    }
}

impl PurchaseOrder {
    pub fn status(&self) -> Status {
        self.status
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
