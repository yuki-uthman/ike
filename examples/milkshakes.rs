use shop::Error;
use shop::Items;
use shop::Loader;
use shop::Tag;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let items =
        Items::load_from_file("assets/Item.csv").map_err(|source| Error::Load { source })?;

    // filter inactive items
    let items: Items = items
        .clone()
        .into_iter()
        .filter(|item| item.is_active())
        .collect::<Vec<_>>()
        .into();

    let milkshake: Items = items
        .find_all("milkshake")?
        .iter_mut()
        .map(|item| {
            item.add_tags(&[Tag::FoodPowder, Tag::Restaurant]);
            item
        })
        .collect::<Vec<_>>()
        .into();

    for item in milkshake.iter() {
        println!("{}", item.name());
    }

    milkshake
        .export("examples/output/milkshake.csv")
        .map_err(|source| Error::Export { source })?;
    Ok(())
}
