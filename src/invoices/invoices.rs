use super::invoice::{Invoice, Status};
use crate::items::{Item, Items};
use crate::loader::Loader;
use crate::ITEMS;
use chrono::NaiveDate as Date;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Invoices {
    invoices: Vec<Invoice>,
}

impl Loader<Invoice> for Invoices {}
impl From<Vec<Invoice>> for Invoices {
    fn from(vec: Vec<Invoice>) -> Invoices {
        Invoices { invoices: vec }
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

impl DerefMut for Invoices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.invoices
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
            .filter(|invoice| invoice.date() >= start)
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

    pub fn get_by_invoice_number(&self, invoice_number: &str) -> Option<&Invoice> {
        self.invoices.iter().find(|invoice| {
            invoice
                .invoice_number()
                .to_lowercase()
                .contains(&invoice_number.to_lowercase())
        })
    }

    pub fn get_closed(&self) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status() == Status::Closed)
            .collect()
    }

    pub fn get_sold(&self) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status() != Status::Draft)
            .collect()
    }

    pub fn count_quantity_sold(&self, item_id: usize) -> usize {
        self.filter_by_item_id(item_id)
            .get_sold()
            .iter()
            .map(|invoice| invoice.quantity())
            .sum()
    }

    pub fn count_frequency(&self, item_id: usize) -> usize {
        self.filter_by_item_id(item_id).len()
    }

    pub fn count_profit_by_item(&self, item_id: usize) -> f32 {
        self.filter_by_item_id(item_id)
            .get_sold()
            .iter()
            .map(|invoice| invoice.profit())
            .sum()
    }

    pub fn count_profit(&self) -> f32 {
        self.get_sold().iter().map(|invoice| invoice.profit()).sum()
    }

    pub fn unique_items(&self) -> Items {
        let mut items = Vec::new();

        for invoice in &self.invoices {
            if invoice.product_id() == 0 {
                continue;
            }
            let item = Item::from(invoice.clone());
            if items.contains(&item) {
            } else {
                items.push(item);
            }
        }
        if items.is_empty() {
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
            .filter(|invoice| {
                invoice.item_name().to_lowercase() == item_name.clone().into().to_lowercase()
            })
            .collect::<Vec<_>>();

        invoices.last().map(|invoice| invoice.date())
    }

    pub fn first_sold_date(&self, item: &Item) -> Option<Date> {
        let invoices = self
            .invoices
            .iter()
            .filter(|invoice| invoice.product_id() == item.id())
            .collect::<Vec<_>>();

        invoices.first().map(|invoice| invoice.date())
    }

    pub fn filter<F>(&self, f: F) -> Self
    where
        F: Fn(&Invoice) -> bool,
    {
        self.invoices.clone().into_iter().filter(f).collect()
    }

    pub fn filter_by_item_id(&self, item_id: usize) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.product_id() == item_id)
            .collect()
    }

    pub fn filter_by_status(&self, status: Status) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.status() == status)
            .collect()
    }

    /// Some invoices contain items with no name.
    /// This function removes those invoices.
    /// Need to run this before injecting items.
    pub fn filter_unnamed_invoice(&self) -> Self {
        self.invoices
            .clone()
            .into_iter()
            .filter(|invoice| invoice.product_id() != 0)
            .collect()
    }

    pub fn remove_by_item_id(&mut self, item_id: usize) {
        self.invoices
            .retain(|invoice| invoice.product_id() != item_id);
    }

    pub fn remove_by_item_name<S>(&mut self, item_name: S)
    where
        S: Into<String> + Clone,
    {
        self.invoices
            .retain(|invoice| invoice.item_name() != item_name.clone().into());
    }

    pub fn inject_items(&mut self) {
        let invoices = self
            .invoices
            .iter_mut()
            .filter(|invoice| invoice.product_id() != 0);

        for invoice in invoices {
            if let Some(item) = ITEMS.get_by_id(invoice.product_id()) {
                invoice.set_item(item.clone());
            } else {
                panic!("Item not found: {:#?}", invoice.item_name());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let invoices = Invoices::load_from_file("assets/Invoice.csv").unwrap();
        assert_yaml_snapshot!(invoices.len(), @r###"
        ---
        7915
        "###);
    }

    #[test]
    fn test_inject_items() {
        let mut invoices = Invoices::load_from_file("assets/Invoice.csv")
            .unwrap()
            .filter_unnamed_invoice();
        invoices.inject_items();

        let filtered_invoices = invoices.filter_by_item_id(3262759000000079001);
        let item = filtered_invoices.first().unwrap().item_as_ref().unwrap();

        let blockboard = ITEMS.get_by_id(3262759000000079001).unwrap();

        assert_eq!(item, blockboard);
    }
}
