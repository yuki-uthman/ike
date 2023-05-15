use serde::de::DeserializeOwned;

use crate::error::Error;
use crate::result::Result;

pub trait Loader <T: DeserializeOwned> {
    fn load(filename: &'static str) -> Result<Vec<T>> {
        let mut reader = csv::Reader::from_path(filename)
            .map_err(|source| Error::FileNotFound { source, filename })?;
        let mut vec = Vec::new();
        for result in reader.deserialize() {
            let record: T = result.map_err(|source| Error::DeserializeFailed { source })?;
            vec.push(record);
        }
        Ok(vec)
    }
}
