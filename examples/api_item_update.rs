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
    let item = items.get_mut("Aluminum Foil Box 83190 With Lid 1850Ml 1 x 300pcs UA ?")?;
    item.set_name("Aluminum Foil Box 83190 With Lid 1850Ml 1 x 300pcs UAE");

    let result = api.update_item(item).await?;
    println!("{:#?}", result);

    Ok(())
}
