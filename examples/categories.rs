use shop::Category;
use shop::Error;
use shop::Items;
use shop::Loader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items = Items::load("assets/Item.csv").map_err(|source| Error::Load { source })?;
    // filter with categories
    let items: Items = items
        .clone()
        .into_iter()
        .filter(|item| item.is(Category::Disposable))
        .collect::<Vec<_>>()
        .into();
    println!("{:#?}", items);
    Ok(())
}
