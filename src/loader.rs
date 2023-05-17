use serde::de::DeserializeOwned;

use crate::error::Error;
use crate::result::Result;

pub trait Loader<Record: DeserializeOwned> {
    fn load(filename: &'static str) -> Result<Self>
    where
        Self: Sized + From<Vec<Record>>,
    {
        let mut reader = csv::Reader::from_path(filename)
            .map_err(|source| Error::FileNotFound { source, filename })?;
        let mut vec = Vec::new();
        for result in reader.deserialize() {
            let record: Record = result.map_err(|source| Error::DeserializeFailed { source })?;
            vec.push(record);
        }

        Ok(vec.into())
    }
}
