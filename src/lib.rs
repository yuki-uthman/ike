mod error;

mod loader;
use loader::Loader;


mod zoho;
use zoho::Invoices;
use zoho::Items;

mod revision;
use revision::Inventories;

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{filename}: {source}")]
    LoadFailed {
        filename: &'static str,
        source: loader::Error,
    },
}

pub fn run() -> std::result::Result<(), Error> {
    let items = Items::load("assets/zoho/Item.csv").map_err(|source| Error::LoadFailed {
        filename: "assets/zoho/Item.csv",
        source,
    })?;

    let inventories = Inventories::load("assets/revision/Inventory.csv").map_err(|source| Error::LoadFailed {
        filename: "assets/revision/Inventory.csv",
        source,
    })?;
    let invoices = Invoices::load("assets/zoho/Invoice.csv").map_err(|source| Error::LoadFailed {
        filename: "assets/zoho/Invoice.csv",
        source,
    })?;

    Ok(())
}
