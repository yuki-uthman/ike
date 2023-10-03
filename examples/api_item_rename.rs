use shop::api::Api;
use shop::Items;
use shop::Loader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = Api::new("credentials".to_string())?;

    if api.token_is_expired() {
        api.refresh_access_token().await;
    }

    let mut items = Items::load_from_file("assets/Item.csv")?;
    items = items.find_all(" plastic")?;

    for item in items.iter() {
        println!("{}", item.name());
    }

    items.replace_string("Round", "Plastic");

    for item in items.iter() {
        println!("{}", item.name());
    }

    for item in items.iter() {
        // let result = api.update_item(item).await?;
        // println!("{:#?}", result);
    }

    Ok(())
}
