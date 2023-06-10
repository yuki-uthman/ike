use colored::Colorize;

mod loader;
pub use loader::Loader;

mod invoices;
pub use invoices::{Invoice, Invoices};

mod items;
pub use items::{Item, Items, Tag};

mod inventories;
pub use inventories::{Inventories, Inventory};

pub type Result<T> = std::result::Result<T, Error>;

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

pub struct Shop {
    items: Items,
    inventories: Inventories,
    invoices: Invoices,
}

impl Shop {
    pub fn new() -> Result<Shop> {
        let items = Items::load_from_file("assets/Item.csv").map_err(|source| Error::Load { source })?;
        let inventories =
            Inventories::load_from_file("assets/Inventory.csv").map_err(|source| Error::Load { source })?;
        let invoices =
            Invoices::load_from_file("assets/Invoice.csv").map_err(|source| Error::Load { source })?;

        Ok(Shop {
            items,
            inventories,
            invoices,
        })
    }

    pub fn items(&self) -> &Items {
        &self.items
    }

    pub fn update_inventories(&mut self) -> Result<()> {
        for inventory in &mut self.inventories.iter() {
            let name = inventory.name();
            let counted_date = inventory.date();
            let quantity = inventory.quantity();
            println!("{}", name.green().bold());
            println!(
                "{}: {}",
                counted_date.to_string().green(),
                quantity.to_string().green().bold()
            );
            let today = chrono::Local::now().date_naive();
            let new_quantity = inventory.quantity()
                - self
                    .invoices
                    .between(counted_date, today)
                    .count_quantity_sold(name);

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

pub fn run() -> Result<()> {
    femme::with_level(femme::LevelFilter::Trace);

    let mut shop = Shop::new()?;
    shop.update_inventories()?;

    Ok(())
}
