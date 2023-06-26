use std::ops::Deref;

// use super::error::Error;
use super::invoice::{Invoice, Status};
use chrono::NaiveDate as Date;
use colored::Colorize;

use crate::items::{Item, Items};
use crate::loader::Loader;

#[derive(Debug)]
pub struct Invoices {
    invoices: Vec<Invoice>,
}

impl Loader<Invoice> for Invoices {}
impl From<Vec<Invoice>> for Invoices {
    fn from(vec: Vec<Invoice>) -> Invoices {
        Invoices {
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

impl Deref for Invoices {
    type Target = Vec<Invoice>;

    fn deref(&self) -> &Self::Target {
        &self.invoices
    }
}

impl Invoices {
    /// Returns a vector of invoices after the given date,
    /// excluding the given date.
    pub fn after(&self, date: Date) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date() > date)
            .collect()
    }

    pub fn before(&self, date: Date) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date() < date)
            .collect()
    }

    pub fn between(&self, start: Date, end: Date) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date() > start)
            .filter(|invoice| invoice.date() <= end)
            .collect()
    }

    pub fn on(&self, date: Date) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.date() == date)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.invoices.len()
    }

    pub fn get_closed(&self) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status() == Status::Closed)
            .collect()
    }

    pub fn count_quantity_sold(&self, product: &str) -> usize {
        let filtered_invoices = self
            .invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status() == Status::Closed)
            .filter(|invoice| invoice.item_name() == product);

        let mut count: usize = 0;
        for invoice in filtered_invoices {
            if invoice.item_name() == product {
                count += invoice.quantity();
            }
        }
        count
    }

    pub fn count_frequency(&self, product: &str) -> usize {
        let filtered_invoices = self
            .invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.item_name() == product);

        filtered_invoices.count()
    }

    pub fn unique_items(&self) -> Items {
        let mut items = Vec::new();

        for invoice in &self.invoices {
            let item = Item::new(&invoice.item_name().clone());
            if items.contains(&item) {
            } else {
                items.push(item);
            }
        }
        if items.len() == 0 {
            panic!("No products found");
        }
        items.into()
    }

    pub fn last_sold<S>(&self, item_name: S) -> Option<Date>
    where
        S: Into<String> + Clone,
    {
        let invoices = self
            .invoices
            .iter()
            .filter(|invoice| invoice.item_name().to_lowercase() == item_name.clone().into().to_lowercase())
            .collect::<Vec<_>>();

        if let Some(item) = invoices.last() {
            Some(item.date().clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let invoices = Invoices::load("assets/Invoice.csv").unwrap();
        assert_yaml_snapshot!(invoices.len(), @r###"
        ---
        5922
        "###);
    }
}
