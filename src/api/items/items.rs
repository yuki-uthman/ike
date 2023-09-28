#![allow(unused)]
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct CustomField {
    label: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct Item {
    item_id: String,
    name: String,
    status: String,
    description: String,
    rate: f64,
    unit: String,
    custom_fields: Vec<CustomField>,
}

#[derive(Deserialize, Debug)]
struct PageContext {
    page: usize,
    per_page: usize,
    has_more_page: bool,
    report_name: String,
    sort_column: String,
    sort_order: String,
}

#[derive(Deserialize, Debug)]
struct ItemsResponse {
    items: Vec<Item>,
    page_context: PageContext,
}

pub struct Items {}
