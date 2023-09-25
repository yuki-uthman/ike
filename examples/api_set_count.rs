use colored::Colorize;
use shop::api::Api;
use shop::Error;
use shop::Invoices;
use shop::Items;
use shop::Loader;
use shop::PurchaseOrders;
use shop::Tag;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let should_update = std::env::args().nth(1);

    let _invoices =
        Invoices::load_from_file("assets/Invoice.csv").map_err(|source| Error::Load { source })?;

    let _purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv")
        .map_err(|source| Error::Load { source })?;

    let mut items = Items::load_from_file("assets/Item.csv")
        .map_err(|source| Error::Load { source })?
        .get_active_items()
        .get_uncounted_items();

    items = items.find_all("toothpick").unwrap();
    items.remove_with_name("5122 laundry basket");
    items.remove_with_name("3 TIRE UTILITY SHELF  VC-0003");
    items.remove_with_name("RD8A SHELF 900X350MM WITH CLIP ONLY");

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
            item.add_tag(Tag::Counted);
            let result = api.update_item(item).await?;
            println!("{:#?}", result);
        }
    }

    Ok(())
}
