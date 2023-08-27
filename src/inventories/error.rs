#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("item not found: {name}")]
    InventoryNotFound { name: String },
}
