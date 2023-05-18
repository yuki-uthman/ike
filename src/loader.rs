use serde::de::DeserializeOwned;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}: {filename}")]
    FileNotFound {
        filename: &'static str,
        source: csv::Error,
    },

    #[error("{source}")]
    DeserializeFailed { source: csv::Error },
}


pub trait Loader<Record: DeserializeOwned> {
    fn load(filename: &'static str) -> Result<Self, Error>
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
