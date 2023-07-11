use online::check;
use super::client::Client;
use super::token::Token;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}")]
    NotInitialized { source: std::io::Error },

    #[error("No internet connection")]
    NoInternetConnection,
}

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

        let client = Client::read_from(&format!("{}/client.json", config)).unwrap();
        let token = Token::read_from(&format!("{}/token.json", config)).map_err(|source| Error::NotInitialized { source })?;

        Ok(Self { config, client, token })
    }

    pub fn token(&self) -> &Token {
        &self.token
    }

    pub async fn refresh_access_token(&mut self) {
        let new_access_token = self.client.get_new_access_token(&self.token.refresh_token()).await.unwrap();
        self.token.set_access_token(new_access_token);

        let token_file = format!("{}/token.json", self.config);
        self.token.write_to(&token_file).unwrap();
    }
}
