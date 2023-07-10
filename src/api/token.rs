use serde::{Deserialize, Serialize};
use std::io::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Token {
    access_token: String,
    refresh_token: String,
    api_domain: String,
}

impl Token {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub fn api_domain(&self) -> &str {
        &self.api_domain
    }

    pub fn set_access_token(&mut self, new_token: String) {
        self.access_token = new_token;
    }

    pub fn read_from(filename: &str) -> Result<Self> {
        let file = std::fs::File::open(filename)?;
        let token = serde_json::from_reader(file)?;
        Ok(token)
    }

    pub fn write_to(&self, filename: &str) -> Result<()> {
        let file = std::fs::File::create(filename)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

}
