#![allow(unused)]
use crate::Item;
use online::check;

use super::client::Client;
use super::error::Error;
use super::items::Item as ApiItem;
use super::token::Token;

#[derive(Debug)]
pub struct Api {
    config: String,

    client: Client,
    token: Token,
}

impl Api {
    pub fn new(config: String) -> Result<Self, Error> {
        if !check(Some(5)).is_ok() {
            return Err(Error::NoInternetConnection);
        }

        let client = Client::from_file(&format!("{}/client.json", config)).unwrap();
        let token = Token::from_file(&format!("{}/token.json", config))
            .map_err(|_| Error::NotInitialized)?;

        Ok(Self {
            config,
            client,
            token,
        })
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn token_is_expired(&self) -> bool {
        self.token.is_expired()
    }

    pub async fn refresh_access_token(&mut self) {
        let new_access_token = self
            .client
            .get_new_access_token(&self.token.refresh_token())
            .await
            .unwrap();
        self.token.set_access_token(new_access_token);
        self.token.renew_time_stamp();

        let token_file = format!("{}/token.json", self.config);
        self.token.write_to(&token_file).unwrap();
    }

    pub async fn update_item(&self, item: &Item) -> Result<(), Error> {
        #[derive(serde::Deserialize, Debug)]
        struct Response {
            code: i32,
            message: String,
            item: ApiItem,
        }

        let item_id = item.id();

        let item = ApiItem::from(item);

        let result = reqwest::Client::new()
            .put(format!(
                "https://www.zohoapis.com/books/v3/items/{}",
                item_id
            ))
            .header(
                "Authorization",
                &format!("Zoho-oauthtoken {}", self.token.access_token()),
            )
            .query(&[("organization_id", &String::from("780294706"))])
            .json(&item)
            .send()
            .await
            .unwrap()
            .json::<Response>()
            .await
            .unwrap();

        println!("{:#?}", result);

        Ok(())
    }
}
