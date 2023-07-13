use serde::{Deserialize, Serialize};
use std::io::Result;
use chrono::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct Token {
    access_token: String,
    refresh_token: String,
    api_domain: String,
    token_type: String,
    expires_in: i64,
    time_stamp: DateTime<Utc>,
}

impl From<serde_json::Value> for Token {
    fn from(object: serde_json::Value) -> Self {
        Self {
            access_token: object.get("access_token").unwrap().to_string(),
            refresh_token: object.get("refresh_token").unwrap().to_string(),
            api_domain: object.get("api_domain").unwrap().to_string(),
            token_type: object.get("token_type").unwrap().to_string(),
            expires_in: object.get("expires_in").unwrap().to_string().parse::<i64>().unwrap(),
            time_stamp: Utc::now(),
        }
    }
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

    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let time_elapsed = now.signed_duration_since(self.time_stamp).num_seconds();
        time_elapsed < self.expires_in
    }

    /// calculates how many seconds left before current token expires
    pub fn expires_in(&self) -> i64 {
        let time_elapsed = Utc::now().signed_duration_since(self.time_stamp).num_seconds();
        if time_elapsed < self.expires_in {
            self.expires_in - time_elapsed
        } else {
            0
        }
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
