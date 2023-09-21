use chrono::NaiveDate;
use shop::api::Api;
use shop::Error;
use shop::Invoices;
use shop::Items;
use shop::Loader;
use shop::PurchaseOrders;
use shop::Tag;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoices =
        Invoices::load_from_file("assets/Invoice.csv").map_err(|source| Error::Load { source })?;

    let purchase_orders = PurchaseOrders::load_from_file("assets/Purchase_Order.csv")
        .map_err(|source| Error::Load { source })?;

    let mut items =
        Items::load_from_file("assets/Item.csv").map_err(|source| Error::Load { source })?;

    items.set_created_date(&purchase_orders, &invoices);

    items = items.created_on(NaiveDate::from_ymd_opt(2023, 7, 30).unwrap());
    items.remove_with_name("Sponge 1010 1x60 pcs");
    items.remove_with_name("GN PAN 1/2");
    items.remove_with_name("Spice Grinder2000W");
    // items = items
    //     .find_all("tray|bed sheet|measuring|pillow|gas|burner|manual|cotton|scale")
    //     .unwrap();

    let today = chrono::Local::now().date_naive();
    let sold_items = invoices
        .between(NaiveDate::from_ymd_opt(2023, 7, 30).unwrap(), today)
        .unique_items();

    for item in items.iter() {
        if sold_items.contains(item) {
            println!("{} is sold", item.name());
        }
    }

    let mut api = Api::new("credentials".to_string())?;

    if api.token_is_expired() {
        api.refresh_access_token().await;
    }

    for item in items.iter_mut() {
        item.add_tag(Tag::Counted);
        let result = api.update_item(item).await?;
        println!("{:#?}", result);
    }

    Ok(())
}
