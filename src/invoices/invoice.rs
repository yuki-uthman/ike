use chrono::NaiveDate as Date;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Draft,
    Closed,
    Overdue,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Invoice {
    #[serde(rename = "Invoice Date", deserialize_with = "deserialize_date")]
    date: Date,
    #[serde(rename = "Invoice Status", deserialize_with = "deserialize_status")]
    status: Status,
    #[serde(rename = "Item Name")]
    item_name: String,
    #[serde(rename = "Quantity")]
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
        "Closed" => Ok(Status::Closed),
        "Overdue" => Ok(Status::Overdue),
        _ => Err(serde::de::Error::custom("invalid status")),
    }
}

impl Invoice {
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
}
