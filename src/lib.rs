use std::result::Result;

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

struct Shop {
    items: Items,
    inventories: Inventories,
    invoices: Invoices,
}

impl Shop {
    fn new() -> Result<Shop, Error> {
        let items = Items::load("assets/zoho/Item.csv").map_err(|source| Error::LoadFailed {
            filename: "assets/zoho/Item.csv",
            source,
        })?;
        let inventories = Inventories::load("assets/revision/Inventory.csv").map_err(|source| {
            Error::LoadFailed {
                filename: "assets/revision/Inventory.csv",
                source,
            }
        })?;
        let invoices =
            Invoices::load("assets/zoho/Invoice.csv").map_err(|source| Error::LoadFailed {
                filename: "assets/zoho/Invoice.csv",
                source,
            })?;

        Ok(Shop {
            items,
            inventories,
            invoices,
        })
    }
}

pub fn run() -> Result<(), Error> {
    let shop = Shop::new()?;

    println!("items: {}", shop.items.len());
    println!("invoices: {}", shop.invoices.len());

    println!("inventories: {}", shop.inventories.len());

    Ok(())
}
