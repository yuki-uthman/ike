use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{source}: {filename}")]
    FileNotFound {
        source: csv::Error,
        filename: &'static str,
    },

    #[error("{source}")]
    DeserializeFailed { source: csv::Error },
}
