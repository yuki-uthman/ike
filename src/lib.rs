use colored::Colorize;
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

    #[error("{source}")]
    Export {
        filename: &'static str,
        source: zoho::items::Error,
    },
}

pub struct Shop {
    items: Items,
    inventories: Inventories,
    invoices: Invoices,
}

impl Shop {
    pub fn new() -> Result<Shop, Error> {
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

    pub fn items(&self) -> &Items {
        &self.items
    }

    fn inventories(&self) -> &Inventories {
        &self.inventories
    }

    fn invoices(&self) -> &Invoices {
        &self.invoices
    }

    pub fn update_inventories(&mut self) -> Result<(), Error> {
        for inventory in &mut self.inventories.iter() {
            let name = inventory.name();
            let date = inventory.date();
            let quantity = inventory.quantity();
            println!("{}", name.green().bold());
            println!(
                "{}: {}",
                date.to_string().green(),
                quantity.to_string().green().bold()
            );
            let new_quantity = inventory.quantity() - self.invoices.set_date(date).count(name);

            self.items
                .get_mut(name)
                .map_err(|source| Error::UpdateInventory { source })?
                .set_quantity(new_quantity);
            println!("     {}: {}\n", "Today".green().bold(), new_quantity.to_string().green().bold());
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
