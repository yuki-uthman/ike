
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Api not initialized")]
    NotInitialized,

    #[error("No internet connection")]
    NoInternetConnection,
}

