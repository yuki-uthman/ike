use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PurchaseOrder {
    #[serde(rename = "Purchase Order Status")]
    status: String,
    #[serde(rename = "Purchase Order Date")]
    date: String,
    #[serde(rename = "Item Name")]
    item_name: String,
    #[serde(rename = "QuantityOrdered")]
    quantity: usize,
}
