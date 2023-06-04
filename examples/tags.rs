use shop::Tag;
use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = Items::load("assets/Item.csv").map_err(|source| Error::Load { source })?;

    // filter with tags
    let items: Items = items
        .clone()
        .into_iter()
        .filter(|item| item.tagged(Tag::Disposable))
        .collect::<Vec<_>>()
        .into();
    println!("{:#?}", items);

    items.export("assets/Item.csv").map_err(|source| Error::Export { source })?;
    Ok(())
}
