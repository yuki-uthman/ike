use serde::{Deserialize, Serialize};
use std::io::Result;
use std::time::SystemTime;

#[derive(Deserialize, Serialize, Debug)]
pub struct Token {
    access_token: String,
    refresh_token: String,
    api_domain: String,
    token_type: String,

    #[serde(skip_serializing, skip_deserializing, default = "SystemTime::now")]
    time_stamp: SystemTime,
    expires_in: i64,
}

impl From<serde_json::Value> for Token {
    fn from(object: serde_json::Value) -> Self {
        Self {
            access_token: object.get("access_token").unwrap().to_string(),
            refresh_token: object.get("refresh_token").unwrap().to_string(),
            api_domain: object.get("api_domain").unwrap().to_string(),
            token_type: object.get("token_type").unwrap().to_string(),
            time_stamp: SystemTime::now(),
            expires_in: object.get("expires_in").unwrap().to_string().parse::<i64>().unwrap(),
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
