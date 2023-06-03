use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Category {
    Disposable,
    Construction,
    Household,
    Office,
    Retail,
    Restaurant,
    Aluminium,
    Steel,
    Plastic,
    Paper,
    Glass,
    Baggase,
    Wood,
    PackagedFood,
    FoodPowder,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "Item Name")]
    name: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "SKU")]
    sku: String,
    #[serde(rename = "Usage unit")]
    usage_unit: String,

    #[serde(rename = "Rate", deserialize_with = "trim_currency")]
    price: f32,
    #[serde(rename = "Purchase Rate", deserialize_with = "trim_currency")]
    cost: f32,
    #[serde(
        skip_deserializing,
        default = "reset_quantity",
        rename(serialize = "Initial Stock")
    )]
    quantity: usize,

    #[serde(rename = "Product Type")]
    product_type: String,
    #[serde(rename = "Item Type")]
    item_type: String,

    #[serde(rename = "Account")]
    account: String,
    #[serde(rename = "Purchase Account")]
    purchase_account: String,
    #[serde(rename = "Inventory Account")]
    inventory_account: String,

    #[serde(rename = "Tax Name")]
    tax_name: String,
    #[serde(rename = "Tax Type")]
    tax_type: String,
    #[serde(rename = "Tax Percentage")]
    tax_percentage: String,

    #[serde(rename = "CF.categories", deserialize_with = "process_categories")]
    categories: HashSet<Category>,
}

fn reset_quantity() -> usize {
    0
}

/// price and cost fields are in the format "MVR 1.00"
/// this function trims the "MVR " prefix and parses the rest as f32
fn trim_currency<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    let s = string.trim_start_matches("MVR ");
    Ok(s.parse().map_err(serde::de::Error::custom)?)
}

fn process_categories<'de, D>(deserializer: D) -> std::result::Result<HashSet<Category>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let result = String::deserialize(deserializer);
    // if string is not empty, split it by comma and parse each category
    // else return an empty vector
    if result.is_err() {
        return Ok(HashSet::new());
    }

    let string = result.unwrap();
    if string.is_empty() {
        return Ok(HashSet::new());
    }

    let categories: HashSet<Category> = string
        .split(',')
        .map(|s| s.trim())
        .map(|s| match s {
            "disposable" => Category::Disposable,
            "construction" => Category::Construction,
            "household" => Category::Household,
            "office" => Category::Office,
            "retail" => Category::Retail,
            "restaurant" => Category::Restaurant,
            "aluminium" => Category::Aluminium,
            "steel" => Category::Steel,
            "plastic" => Category::Plastic,
            "paper" => Category::Paper,
            "glass" => Category::Glass,
            "baggase" => Category::Baggase,
            "wood" => Category::Wood,
            "packaged food" => Category::PackagedFood,
            "food powder" => Category::FoodPowder,
            _ => panic!("unknown category: {}", s),
        })
    .collect();
    Ok(categories)
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Item {
    pub fn new(name: &str) -> Self {
        Self {
            status: "Active".to_string(),
            name: name.to_string(),
            description: "".to_string(),
            sku: "".to_string(),
            usage_unit: "pcs".to_string(),
            price: 0.0,
            cost: 0.0,
            quantity: 0,
            product_type: "goods".to_string(),
            item_type: "inventory".to_string(),
            account: "Inventory Assets".to_string(),
            purchase_account: "Cost of Goods Sold".to_string(),
            inventory_account: "Inventory Assets".to_string(),
            tax_name: "".to_string(),
            tax_type: "".to_string(),
            tax_percentage: "".to_string(),
            categories: HashSet::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn price(&self) -> f32 {
        self.price
    }

    pub fn cost(&self) -> f32 {
        self.cost
    }

    pub fn quantity(&self) -> usize {
        self.quantity
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn set_price(&mut self, price: f32) -> &mut Self {
        self.price = price;
        self
    }

    pub fn set_cost(&mut self, cost: f32) -> &mut Self {
        self.cost = cost;
        self
    }

    pub fn set_quantity(&mut self, quantity: usize) -> &mut Self {
        self.quantity = quantity;
        self
    }

    pub fn is_active(&self) -> bool {
        self.status == "Active"
    }

    pub fn is(&self, category: Category) -> bool {
        self.categories.contains(&category)
    }
}
