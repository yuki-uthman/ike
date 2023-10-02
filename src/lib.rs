// #![allow(clippy::correctness)]
// #![allow(clippy::suspicious)]
// #![allow(clippy::complexity)]
// #![allow(clippy::perf)]
// #![allow(clippy::style)]
// #![allow(clippy::pedantic)]
// #![allow(clippy::restriction)]

#![allow(clippy::module_inception)]
#![allow(clippy::len_without_is_empty)]

mod loader;
pub use loader::Loader;

mod invoices;
pub use invoices::Status as InvoiceStatus;
pub use invoices::{Invoice, Invoices};

mod items;
pub use items::TaxName as ItemTaxName;
pub use items::{Item, Items, Tag, Tags};

mod inventories;
pub use inventories::{Inventories, Inventory};

mod groups;
pub use groups::{Group, Groups};

mod purchase_orders;
pub use purchase_orders::Status as PurchaseOrderStatus;
pub use purchase_orders::{PurchaseOrder, PurchaseOrders};

pub mod api;

pub type Result<T> = std::result::Result<T, Error>;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref ITEMS: Items = Items::load_from_file("assets/Item.csv").unwrap();
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}")]
    Load { source: loader::Error },
    #[error("{source}")]
    UpdateInventory { source: items::Error },

    #[error("{source}")]
    Export { source: items::Error },
}
