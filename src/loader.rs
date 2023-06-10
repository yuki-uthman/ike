use serde::de::DeserializeOwned;
use std::fs::read_dir;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}: {filename}")]
    FileNotFound {
        filename: &'static str,
        source: csv::Error,
    },

    #[error("{source}: {dir}")]
    DirectoryNotFound {
        dir: &'static str,
        source: std::io::Error,
    },

    #[error("{source}")]
    DirectoryIteration { source: std::io::Error },

    #[error("{source}")]
    DeserializeFailed { source: csv::Error },
}

pub trait Loader<Record: DeserializeOwned> {
    #[deprecated(note = "Use `load_from_file` instead")]
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

    fn load_from_file(filename: &'static str) -> Result<Self, Error>
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

    fn load_from_dir(dir: &'static str) -> Result<Self, Error>
    where
        Self: Sized + From<Vec<Record>>,
    {
        let mut vec = Vec::new();
        for entry in
            std::fs::read_dir(dir).map_err(|source| Error::DirectoryNotFound { source, dir })?
        {
            let path = entry.map_err(|source| Error::DirectoryIteration { source })?.path();
            if path.is_file() {
                let mut reader =
                    csv::Reader::from_path(path).map_err(|source| Error::FileNotFound {
                        source,
                        filename: dir,
                    })?;
                for result in reader.deserialize() {
                    let record: Record =
                        result.map_err(|source| Error::DeserializeFailed { source })?;
                    vec.push(record);
                }
            }
        }
        Ok(vec.into())
    }
}
