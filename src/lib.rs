use colored::Colorize;
use std::result::Result;

mod loader;
pub use loader::Loader;

mod invoices;
pub use invoices::{Invoice, Invoices};

mod items;
pub use items::{Item, Items, Tag};

mod inventories;
pub use inventories::{Inventories, Inventory};

use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}")]
    Load { source: loader::Error },
    #[error("{source}")]
    UpdateInventory { source: items::Error },

    #[error("{source}")]
    Export {
        source: items::Error,
    },
}

pub struct Shop {
    items: Items,
    inventories: Inventories,
    invoices: Invoices,
}

impl Shop {
    pub fn new() -> Result<Shop, Error> {
        let items = Items::load("assets/Item.csv").map_err(|source| Error::Load { source })?;
        let inventories = Inventories::load("assets/Inventory.csv")
            .map_err(|source| Error::Load { source })?;
        let invoices =
            Invoices::load("assets/Invoice.csv").map_err(|source| Error::Load { source })?;

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
            let new_quantity =
                inventory.quantity() - self.invoices.set_date(date).count_quantity_sold(name);

            self.items
                .get_mut(name)
                .map_err(|source| Error::UpdateInventory { source })?
                .set_quantity(new_quantity);
            println!(
                "     {}: {}\n",
                "Today".green().bold(),
                new_quantity.to_string().green().bold()
            );
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
