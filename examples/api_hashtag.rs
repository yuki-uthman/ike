#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use colored::Colorize;
use shop::api::Api;
use shop::Error;
use shop::Invoices;
use shop::Items;
use shop::Loader;
use shop::Tag;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let should_update = std::env::args().nth(1);

    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_tagged_items();

    // items = items.find_all("73365").unwrap();

    for item in items.iter() {
        println!("   {}", item.name().green().bold());
    }

    let mut api = Api::new("credentials".to_string())?;

    // if should_update is true, update the items in the database
    if should_update.is_some() {
        if api.token_is_expired() {
            api.refresh_access_token().await;
        }

        for item in items.iter_mut() {
            // if tag contains other than counted
            if item.tags().contains(&Tag::Aluminium) {
                let result = api.update_item(item).await?;
                println!("{:#?}", result);
            } else if item.tags().contains(&Tag::Disposable) {
                let result = api.update_item(item).await?;
                println!("{:#?}", result);
            }
        }
    }

    Ok(())
}
