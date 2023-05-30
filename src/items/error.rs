
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("item not found: {name}")]
    ItemNotFound { name: String },

    #[error("no items with the keyword: {keyword}")]
    NoSuchItems { keyword: String },

    #[error("{source}")]
    FileCreate {
        filename: &'static str,
        source: std::io::Error,
    },

    #[error("{source}")]
    FileOpen {
        filename: &'static str,
        source: csv::Error,
    },

    #[error("{source}")]
    Serialization { source: csv::Error },

    #[error("{source}")]
    Flush { source: std::io::Error },

    #[error("{index}")]
    IndexOutOfBounds { index: usize },
}
