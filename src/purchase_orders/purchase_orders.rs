use super::PurchaseOrder;
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

impl PurchaseOrders {
    pub fn between(&self, start: Date, end: Date) -> impl Iterator<Item = PurchaseOrder> {
        self.0
            .clone()
            .into_iter()
            .filter(move |po| po.date() >= start && po.date() <= end)
    }
        self.0
            .clone()
            .into_iter()
            .filter(|po| po.date() >= start && po.date() <= end)
            .collect()
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
        444
        "###);
    }
}
