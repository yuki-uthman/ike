use online::check;
use super::client::Client;
use super::token::Token;
use super::error::Error;

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
        let token = Token::from_file(&format!("{}/token.json", config)).map_err(|_| Error::NotInitialized )?;

        Ok(Self { config, client, token })
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub fn token_is_expired(&self) -> bool {
        self.token.is_expired()
    }

    pub async fn refresh_access_token(&mut self) {
        let new_access_token = self.client.get_new_access_token(&self.token.refresh_token()).await.unwrap();
        self.token.set_access_token(new_access_token);

        let token_file = format!("{}/token.json", self.config);
        self.token.write_to(&token_file).unwrap();
    }
}
