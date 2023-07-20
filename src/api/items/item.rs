use crate::Item as FileItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomField {
    label: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    custom_fields: Vec<CustomField>,
    name: String,
}


impl From<&FileItem> for Item {
    fn from(item: &FileItem) -> Self {

        let tags = CustomField {
            label: "tags".to_string(),
            value: item.tags().to_string(),
        };

        Self {
            name: item.name().to_string(),
            custom_fields: vec![tags]
        }
    }
}
