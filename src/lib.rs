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
    Load {
        filename: &'static str,
        source: loader::Error,
    },
    #[error("{source}")]
    UpdateInventory { source: zoho::items::Error },
}

struct Shop {
    items: Items,
    inventories: Inventories,
    invoices: Invoices,
}

impl Shop {
    fn new() -> Result<Shop, Error> {
        let items = Items::load("assets/zoho/Item.csv").map_err(|source| Error::Load {
            filename: "assets/zoho/Item.csv",
            source,
        })?;
        let inventories = Inventories::load("assets/revision/Inventory.csv").map_err(|source| {
            Error::Load {
                filename: "assets/revision/Inventory.csv",
                source,
            }
        })?;
        let invoices =
            Invoices::load("assets/zoho/Invoice.csv").map_err(|source| Error::Load {
                filename: "assets/zoho/Invoice.csv",
                source,
            })?;

        Ok(Shop {
            items,
            inventories,
            invoices,
        })
    }

    fn items(&self) -> &Items {
        &self.items
    }

    fn inventories(&self) -> &Inventories {
        &self.inventories
    }

    fn invoices(&self) -> &Invoices {
        &self.invoices
    }

    fn update_inventories(&mut self) -> Result<(), Error> {
        for inventory in &mut self.inventories.iter() {
            let name = inventory.name();
            let date = inventory.date();

            let quantity = self.invoices.set_date(date).count(name);

            self.items
                .get_mut(name)
                .map_err(|source| Error::UpdateInventory { source })?
                .set_quantity(quantity);
        }
        Ok(())
    }
}

pub fn run() -> Result<(), Error> {
    femme::with_level(femme::LevelFilter::Trace);

    let mut shop = Shop::new()?;
    shop.update_inventories()?;

    Ok(())
}
