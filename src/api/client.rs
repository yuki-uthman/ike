use super::Token;
use serde::Deserialize;

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct Client {
    id: String,
    secret: String,
}

impl Client {
    pub fn from_file(filename: &str) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(filename)?;
        let client: Client = serde_json::from_reader(file)?;

        Ok(client)
    }

    pub async fn get_initial_token(&self, code: &str) -> Result<Token, reqwest::Error> {
        let response = reqwest::Client::new()
            .post("https://accounts.zoho.com/oauth/v2/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("client_id", &self.id),
                ("client_secret", &self.secret),
            ])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if response.get("error").is_some() {
            panic!("Error: {:#?}", response);
        }

        Ok(Token::from(response))
    }

    pub async fn get_new_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<String, reqwest::Error> {
        #[derive(Deserialize, Debug)]
        struct RefreshResponse {
            access_token: String,
        }

        let result = reqwest::Client::new()
            .post("https://accounts.zoho.com/oauth/v2/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.id),
                ("client_secret", &self.secret),
            ])
            .send()
            .await?
            .json::<RefreshResponse>()
            .await?;

        Ok(result.access_token)
    }
}
