use super::error::Error;
use super::item::Item;
use crate::{loader::Loader, Invoices, PurchaseOrders};
use chrono::NaiveDate;
use regex::RegexBuilder;
use std::{
    collections::HashSet,
    ops::{Add, Deref, DerefMut, Sub},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Loader<Item> for Items {}
impl From<Vec<Item>> for Items {
    fn from(vec: Vec<Item>) -> Items {
        Items(vec)
    }
}

impl From<Vec<&Item>> for Items {
    fn from(vec: Vec<&Item>) -> Items {
        Items(vec.into_iter().cloned().collect())
    }
}

impl From<Vec<&mut Item>> for Items {
    fn from(vec: Vec<&mut Item>) -> Items {
        Items(vec.into_iter().map(|item| item.clone()).collect())
    }
}

impl From<Vec<String>> for Items {
    fn from(vec: Vec<String>) -> Items {
        let mut items = Vec::new();
        for name in vec {
            items.push(Item::new(&name));
        }
        items.into()
    }
}

impl From<Items> for HashSet<String> {
    fn from(items: Items) -> HashSet<String> {
        let set = items
            .iter()
            .map(|item| item.name().to_string())
            .collect::<HashSet<String>>();
        set
    }
}

impl Default for Items {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Items {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Items {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add<Items> for Items {
    type Output = Items;

    fn add(mut self, items: Items) -> Self::Output {
        for item in items.0 {
            if !self.0.contains(&item) {
                self.0.push(item);
            }
        }
        self
    }
}

impl Sub<Items> for Items {
    type Output = Items;

    fn sub(mut self, items: Items) -> Self::Output {
        for item in items.0 {
            if let Some(index) = self.0.iter().position(|i| i.name() == item.name()) {
                self.0.remove(index);
            }
        }
        self
    }
}

impl Items {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn replace_string(&mut self, from: &str, to: &str) {
        for item in self.iter_mut() {
            item.replace_string(from, to);
        }
    }

    pub fn set_created_date(&mut self, purchase_orders: &PurchaseOrders, invoices: &Invoices) {
        for item in self.iter_mut() {
            let first_po = purchase_orders.first_bought_date(item);

            let first_invoice = invoices.first_sold_date(item);

            let date = match (first_po, first_invoice) {
                (Some(po), Some(inv)) => {
                    if po < inv {
                        po
                    } else {
                        inv
                    }
                }
                (Some(po), None) => po,
                (None, Some(inv)) => inv,
                (None, None) => continue,
            };

            item.set_created_date(date);
        }
    }

    pub fn created_after(&self, date: NaiveDate) -> Self {
        self.iter()
            .filter(|item| item.created_date() > date)
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn created_on(&self, date: NaiveDate) -> Self {
        self.iter()
            .filter(|item| item.created_date() == date)
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn contains<S>(&self, item: S) -> bool
    where
        S: AsRef<str>,
    {
        self.iter().any(|i| i.name() == item.as_ref())
    }

    pub fn filter<F>(&self, f: F) -> Self
    where
        F: Fn(&Item) -> bool,
    {
        self.iter()
            .filter(|i| f(i))
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_active_items(&self) -> Self {
        self.iter()
            .filter(|item| item.is_active())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_inactive_items(&self) -> Self {
        self.iter()
            .filter(|item| !item.is_active())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_untagged_items(&self) -> Self {
        self.iter()
            .filter(|item| item.group().is_empty())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_tagged_items(&self) -> Self {
        self.iter()
            .filter(|item| item.is_tagged())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_unique_items(&self) -> Self {
        let mut unique_items = Vec::new();
        for item in self.iter() {
            if !unique_items.contains(item) {
                unique_items.push(item.clone());
            }
        }
        unique_items.into()
    }

    pub fn get_counted_items(&self) -> Self {
        self.iter()
            .filter(|item| item.is_counted())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_uncounted_items(&self) -> Self {
        self.iter()
            .filter(|item| !item.is_counted())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_non_combo_items(&self) -> Self {
        self.iter()
            .filter(|item| !item.is_combo_product())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_has_stock_items(&self) -> Self {
        self.iter()
            .filter(|item| item.has_stock())
            .cloned()
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn add(&mut self, item: Item) {
        self.0.push(item);
    }

    pub fn remove_with_name(&mut self, name: &str) {
        if let Some(index) = self.0.iter().position(|i| i.name() == name) {
            self.0.remove(index);
        }
    }

    pub fn find_all(&self, name: &str) -> Result<Self> {
        let mut matches = Vec::new();
        let re = RegexBuilder::new(name)
            .case_insensitive(true)
            .build()
            .unwrap();

        for item in &self.0 {
            if re.is_match(item.name()) || re.is_match(item.group()) {
                matches.push(item.clone());
            }
        }
        if matches.is_empty() {
            return Err(Error::NoSuchItems {
                keyword: name.to_string(),
            });
        }

        let mut items: Items = matches.into();
        items.sort_by_name();

        Ok(items)
    }

    pub fn get(&self, item_name: &str) -> Result<&Item> {
        for item in &self.0 {
            if item.name() == item_name {
                return Ok(item);
            }
        }
        Err(Error::ItemNotFound {
            name: item_name.to_string(),
        })
    }

    pub fn get_mut(&mut self, item_name: &str) -> Result<&mut Item> {
        for item in &mut self.0 {
            if item.name() == item_name {
                return Ok(item);
            }
        }
        Err(Error::ItemNotFound {
            name: item_name.to_string(),
        })
    }

    pub fn get_by_index(&self, index: usize) -> Result<&Item> {
        if index >= self.0.len() {
            return Err(Error::IndexOutOfBounds { index });
        }
        Ok(&self.0[index])
    }

    pub fn get_by_id(&self, id: usize) -> Option<&Item> {
        self.0.iter().find(|item| item.id() == id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn take(&self, how_many: usize) -> Self {
        let items = self.0.iter().take(how_many).cloned().collect::<Vec<_>>();

        items.into()
    }

    pub fn export<S>(&self, filename: S) -> Result<()>
    where
        S: Into<String> + Copy,
    {
        std::fs::File::create(filename.into()).map_err(|source| Error::FileCreate {
            filename: filename.into(),
            source,
        })?;

        let mut writer =
            csv::Writer::from_path(filename.into()).map_err(|source| Error::FileOpen {
                filename: filename.into(),
                source,
            })?;
        for item in &self.0 {
            writer
                .serialize(item)
                .map_err(|source| Error::Serialization { source })?;
        }
        writer.flush().map_err(|source| Error::Flush { source })?;
        Ok(())
    }

    pub fn sort_by_name(&mut self) {
        self.0
            .sort_by_key(|item| item.name().to_lowercase().to_string());
    }
}

#[cfg(test)]
mod tests {
    use crate::Tag;

    use super::*;

    #[test]
    fn load() {
        let res = Items::load_from_file("assets/Item.csv");
        assert!(res.is_ok());
        let items = res.unwrap();

        assert!(items.len() > 0);
    }

    #[test]
    fn replace_string() {
        let item1 = Item::new("item1");
        let item2 = Item::new("item2");

        let mut items = Items::from(vec![item1, item2]);

        items.replace_string("item", "product");

        assert_eq!(items.get_by_index(0).unwrap().name(), "product1");
        assert_eq!(items.get_by_index(1).unwrap().name(), "product2");
    }

    #[test]
    fn get_tagged_items() {
        let item1 = Item::new("item1");
        let item2 = Item::new("item2");
        let mut item3 = Item::new("item3");
        item3.add_tag(Tag::Paper);

        let items = Items::from(vec![item1, item2, item3]).get_tagged_items();

        assert_eq!(items.len(), 1);
    }
}
