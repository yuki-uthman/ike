use super::error::Error;
use super::item::Item;
use crate::loader::Loader;
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
        Items(vec.into_iter().map(|item| item.clone()).collect())
    }
}

impl From<Vec<&mut Item>> for Items {
    fn from(vec: Vec<&mut Item>) -> Items {
        Items(vec.into_iter().map(|item| item.clone()).collect())
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
            .map(|i| i.clone())
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_active_items(&self) -> Self {
        self.iter()
            .filter(|item| item.is_active())
            .map(|item| item.clone())
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_inactive_items(&self) -> Self {
        self.iter()
            .filter(|item| !item.is_active())
            .map(|item| item.clone())
            .collect::<Vec<Item>>()
            .into()
    }

    pub fn get_untagged_items(&self) -> Self {
        self.iter()
            .filter(|item| item.group().is_empty())
            .map(|item| item.clone())
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

    pub fn add(&mut self, item: Item) {
        self.0.push(item);
    }

    pub fn find_all(&self, name: &str) -> Result<Self> {
        let mut matches = Vec::new();
        let re = RegexBuilder::new(name)
            .case_insensitive(true)
            .build()
            .unwrap();

        for item in &self.0 {
            if re.is_match(&item.name()) || re.is_match(&item.group()) {
                matches.push(item.clone());
            }
        }
        if matches.is_empty() {
            return Err(Error::NoSuchItems {
                keyword: name.to_string(),
            });
        }
        matches.sort_by(|a, b| a.name().cmp(&b.name()));
        Ok(matches.into())
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

    pub fn sort(&mut self) {
        self.0
            .sort_by(|a, b| a.name().to_lowercase().cmp(&b.name().to_lowercase()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let items = Items::load("assets/Item.csv").unwrap();
        assert_yaml_snapshot!(items.len(), @r###"
        ---
        718
        "###);
    }
}
