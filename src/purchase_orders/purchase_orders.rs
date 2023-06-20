use super::PurchaseOrder;
use crate::loader::Loader;
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
