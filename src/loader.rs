use serde::de::DeserializeOwned;
use std::fs::read_dir;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}: {filename}")]
    FileNotFound {
        filename: String,
        source: csv::Error,
    },

    #[error("{source}: {dir}")]
    DirectoryNotFound { dir: String, source: std::io::Error },

    #[error("{source}")]
    DirectoryIteration { source: std::io::Error },

    #[error("{source}")]
    DeserializeFailed { source: csv::Error },

    #[error("directory: {dir}")]
    DirectoryEmpty { dir: String },
}

pub trait Loader<Record: DeserializeOwned> {
    #[deprecated(note = "Use `load_from_file` instead")]
    fn load(filename: &'static str) -> Result<Self, Error>
    where
        Self: Sized + From<Vec<Record>>,
    {
        let mut reader =
            csv::Reader::from_path(filename).map_err(|source| Error::FileNotFound {
                source,
                filename: filename.to_string(),
            })?;
        let mut vec = Vec::new();
        for result in reader.deserialize() {
            let record: Record = result.map_err(|source| Error::DeserializeFailed { source })?;
            vec.push(record);
        }

        Ok(vec.into())
    }

    fn load_from_file<S>(filename: S) -> Result<Self, Error>
    where
        Self: Sized + From<Vec<Record>>,
        S: Into<String> + Clone,
    {
        let mut reader = csv::Reader::from_path(filename.clone().into()).map_err(|source| {
            Error::FileNotFound {
                source,
                filename: filename.into(),
            }
        })?;
        let mut vec = Vec::new();
        for result in reader.deserialize() {
            let record: Record = result.map_err(|source| Error::DeserializeFailed { source })?;
            vec.push(record);
        }

        Ok(vec.into())
    }

    fn load_from_dir<S>(dir: S) -> Result<Self, Error>
    where
        Self: Sized + From<Vec<Record>>,
        S: Into<String> + Clone,
    {
        let mut vec = Vec::new();
        let dir_iter = read_dir(dir.clone().into()).map_err(|source| Error::DirectoryNotFound {
            source,
            dir: dir.clone().into(),
        })?;

        for (file_count, entry) in dir_iter.enumerate() {
            let path = entry
                .map_err(|source| Error::DirectoryIteration { source })?
                .path();

            if file_count == 0 && path.file_name().unwrap() == ".DS_Store" {
                return Err(Error::DirectoryEmpty { dir: dir.into() });
            }

            if path.is_file() {
                let mut reader =
                    csv::Reader::from_path(&path).map_err(|source| Error::FileNotFound {
                        source,
                        filename: path.to_str().unwrap().to_string(),
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
