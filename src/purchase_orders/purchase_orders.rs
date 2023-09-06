use super::{purchase_order::Status, PurchaseOrder};
use crate::Item;
use crate::items::Items;
use crate::loader::Loader;
use chrono::NaiveDate as Date;
use std::ops::Deref;

#[derive(Debug)]
pub struct PurchaseOrders(Vec<PurchaseOrder>);

impl Loader<PurchaseOrder> for PurchaseOrders {}

impl From<Vec<PurchaseOrder>> for PurchaseOrders {
    fn from(vec: Vec<PurchaseOrder>) -> PurchaseOrders {
        PurchaseOrders(vec)
    }
}

impl Deref for PurchaseOrders {
    type Target = Vec<PurchaseOrder>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<PurchaseOrder> for PurchaseOrders {
    fn from_iter<I: IntoIterator<Item = PurchaseOrder>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for po in iter {
            vec.push(po);
        }
        vec.into()
    }
}

impl IntoIterator for PurchaseOrders {
    type Item = PurchaseOrder;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a PurchaseOrders {
    type Item = &'a PurchaseOrder;
    type IntoIter = std::slice::Iter<'a, PurchaseOrder>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl PurchaseOrders {
    pub fn between(&self, start: Date, end: Date) -> PurchaseOrders {
        self.0
            .clone()
            .into_iter()
            .filter(move |po| po.date() >= start && po.date() <= end)
            .collect()
    }

    pub fn filter<F>(&self, predicate: F) -> PurchaseOrders
    where
        F: Fn(&PurchaseOrder) -> bool,
    {
        self.0.clone().into_iter().filter(predicate).collect()
    }

    pub fn filter_by_item_id<S>(&self, item_id: S) -> PurchaseOrders 
        where
            S: AsRef<str>,
    {
        self.filter(move |po| po.product_id() == item_id.as_ref())
    }

    pub fn filter_by_item_name<S>(&self, item_name: S) -> PurchaseOrders
    where
        S: AsRef<str>,
    {
        self.filter(move |po| po.item_name() == item_name.as_ref())
    }

    pub fn filter_by_status<S>(&self, status: S) -> PurchaseOrders
    where
        S: Into<Status> + Copy,
    {
        self.filter(|po| po.status() == status.into())
    }

    pub fn into_quantity(self) -> impl Iterator<Item = usize> {
        self.into_iter().map(|po| po.quantity())
    }

    pub fn unique_items(&self) -> Items {
        let mut items = Vec::new();
        for po in self {
            if !items.contains(&po.item_name()) {
                items.push(po.item_name());
            }
        }

        if items.is_empty() {
            panic!("No items in the purchase order");
        }

        items.into()
    }

    pub fn first_bought_date(&self, item: &Item) -> Option<Date>
    {
        self.filter(|po| po.product_id() == item.id())
            .into_iter()
            .map(|po| po.date())
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let po = PurchaseOrders::load_from_file("assets/Purchase_Order.csv").unwrap();
        assert_yaml_snapshot!(po.len(), @r###"
        ---
        574
        "###);
    }
}
