use shop::{Error, Shop};

fn main() -> Result<(), Error> {
    femme::with_level(femme::LevelFilter::Trace);

    let mut shop = Shop::new()?;
    shop.update_inventories()?;
    // shop.update_inventories(filename)?; better?

    shop.items()
        .take(10)
        .export("examples/output/Item.csv")
        .map_err(|source| Error::Export {
            filename: "examples/output/Item.csv",
            source,
        })?;

    Ok(())
}
