use serde::{Deserialize, Serialize};
use crate::Item as FileItem;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomField {
    label : String,
    value : String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    custom_fields : Vec<CustomField>,
    name : String,
}

impl From<&FileItem> for Item {
    fn from(item: &FileItem) -> Self {
        Self {
            name: item.name().to_string(),
            custom_fields: Vec::new(),
        }
    }
}
