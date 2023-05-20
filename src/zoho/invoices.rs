use chrono::NaiveDate as Date;
use kv_log_macro as log;
use serde::Deserialize;

use crate::loader::Loader;

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

#[derive(Debug)]
pub struct Invoices {
    date: Date,
    invoices: Vec<Invoice>,
}

impl Loader<Invoice> for Invoices {}
impl From<Vec<Invoice>> for Invoices {
    fn from(vec: Vec<Invoice>) -> Invoices {
        Invoices {
            date: Date::from_ymd_opt(2020, 1, 1).unwrap(),
            invoices: vec,
        }
    }
}

impl FromIterator<Invoice> for Invoices {
    fn from_iter<I: IntoIterator<Item = Invoice>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for invoice in iter {
            vec.push(invoice);
        }
        vec.into()
    }
}

impl Invoices {
    pub fn set_date(&mut self, date: Date) -> &mut Self {
        log::info!("{}", date);
        self.date = date;
        self
    }

    /// Returns a vector of invoices after the given date,
    /// excluding the given date.
    pub fn after(&self, date: Date) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date > date)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.invoices.len()
    }

    pub fn closed(&self) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status == Status::Closed)
            .collect()
    }

    pub fn count(&self, product: &str) -> usize {
        let filtered_invoices = self
            .invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date > self.date)
            .filter(|invoice| invoice.status == Status::Closed)
            .filter(|invoice| invoice.product == product);

        let mut count: usize = 0;
        log::info!("{}", product);
        for invoice in filtered_invoices {
            log::info!("{}: {}pcs sold", invoice.date, invoice.quantity);
            if invoice.product == product {
                count += invoice.quantity;
            }
        }
        log::info!("     Total: {}pcs sold", count);
        println!();
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let invoices = Invoices::load("tests/assets/zoho/Invoice.csv").unwrap();
        assert_yaml_snapshot!(invoices.len(), @r###"
        ---
        5030
        "###);
    }
}
