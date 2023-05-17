use chrono::NaiveDate as Date;
use serde::Deserialize;

use crate::records::Records;
use crate::result::Result;

#[derive(Clone, Debug, PartialEq)]
enum Status {
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
    product: String,
    #[serde(rename = "Quantity")]
    quantity: usize,
}

fn deserialize_date<'de, D>(deserializer: D) -> std::result::Result<Date, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer).unwrap();
    Ok(Date::parse_from_str(&s, "%Y-%m-%d").unwrap())
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

#[derive(Debug)]
pub struct Invoices(Vec<Invoice>);

impl Records<Invoice> for Invoices {}
impl From<Vec<Invoice>> for Invoices {
    fn from(vec: Vec<Invoice>) -> Invoices {
        Invoices(vec)
    }
}

impl FromIterator<Invoice> for Invoices {
    fn from_iter<I: IntoIterator<Item = Invoice>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for invoice in iter {
            vec.push(invoice);
        }
        Invoices(vec)
    }
}

impl Invoices {
    /// Returns a vector of invoices after the given date,
    /// excluding the given date.
    pub fn after(&self, date: Date) -> Self {
        self.0
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date > date)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn closed(&self) -> Self {
        self.0
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status == Status::Closed)
            .collect()
    }

    pub fn count(&self, product: &str) -> usize {
        let filtered_invoices = self.0.iter().filter(|invoice| invoice.product == product);
        let mut count: usize = 0;
        for invoice in filtered_invoices {
            if invoice.product == product {
                count += invoice.quantity;
            }
        }
        count
    }
}
