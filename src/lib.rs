mod loader;
pub use loader::Loader;

mod invoices;
pub use invoices::{Invoice, Invoices};
pub use invoices::Status as InvoiceStatus;

mod items;
pub use items::{Item, Items, Tags, Tag};
pub use items::TaxName as ItemTaxName;

mod inventories;
pub use inventories::{Inventories, Inventory};

mod groups;
pub use groups::{Group, Groups};

mod purchase_orders;
pub use purchase_orders::{PurchaseOrder, PurchaseOrders};
pub use purchase_orders::Status as PurchaseOrderStatus;

pub mod api;

pub type Result<T> = std::result::Result<T, Error>;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref ITEMS: Items = Items::load_from_file("assets/Item.csv").unwrap();
}

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}")]
    Load { source: loader::Error },
    #[error("{source}")]
    UpdateInventory { source: items::Error },

    #[error("{source}")]
    Export { source: items::Error },
}
