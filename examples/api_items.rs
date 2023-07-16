use std::str::FromStr;

use reqwest;
use serde::Deserialize;

use shop::api::Client;
use shop::api::Token;
use shop::Tags;

#[derive(Deserialize, Debug)]
struct CustomField {
    label: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct Item {
    item_id: String,
    name: String,
    status: String,
    description: String,
    rate: f64,
    unit: String,
    custom_fields: Vec<CustomField>,
}

fn de_tags<'de, D>(deserializer: D) -> std::result::Result<Tags, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let result = String::deserialize(deserializer);
    // if string is not empty, split it by comma and parse each category
    // else return an empty vector
    if result.is_err() {
        return Ok(Tags::new());
    }

    let string = result.unwrap();
    if string.is_empty() {
        return Ok(Tags::new());
    }

    let tags = Tags::from_str(&string).unwrap();
    Ok(tags)
}

#[derive(Deserialize, Debug)]
struct PageContext {
    page: usize,
    per_page: usize,
    has_more_page: bool,
    report_name: String,
    sort_column: String,
    sort_order: String,
}

#[derive(Deserialize, Debug)]
struct ItemResponse {
    item: Item,
}

#[derive(Deserialize, Debug)]
struct ItemID {
    item_id: String,
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
    items: Vec<ItemID>,
    page_context: PageContext,
}

#[tokio::main]
async fn main() {
    let token = Token::from_file("credentials/token.json").unwrap();

    println!("{:#?}", &token);

    if !token.is_valid() {

        println!("Token is not valid");
        return;
    }


    let items: SearchResponse = reqwest::Client::new()
        .get(&format!("{}/books/v3/items", token.api_domain()))
        .header(
            "Authorization",
            &format!("Zoho-oauthtoken {}", token.access_token()),
        )
        .query(&[
            ("organization_id", &String::from("780294706")),
            ("search_text", &String::from("bamboo")),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    for item in items.items {
        let item: ItemResponse = reqwest::Client::new()
            .get(&format!(
                "{}/books/v3/items/{}",
                token.api_domain(),
                item.item_id
            ))
            .header(
                "Authorization",
                &format!("Zoho-oauthtoken {}", token.access_token()),
            )
            .query(&[("organization_id", &String::from("780294706"))])
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        println!("{:#?}", item.item.name);
    }

    // let item: ItemResponse = reqwest::Client::new()
    //     .get(&format!(
    //         "{}/books/v3/items/3262759000002782007",
    //         token.api_domain()
    //     ))
    //     .header(
    //         "Authorization",
    //         &format!("Zoho-oauthtoken {}", token.access_token()),
    //     )
    //     .query(&[("organization_id", &String::from("780294706"))])
    //     .send()
    //     .await
    //     .unwrap()
    //     .json()
    //     .await
    //     .unwrap();

    // println!("{:#?}", item);
}
