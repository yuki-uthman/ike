use super::error::Error;
use super::item::Item;
use crate::loader::Loader;
use regex::RegexBuilder;
use std::ops::{Add, Deref, Sub};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Items(Vec<Item>);

impl Loader<Item> for Items {}
impl From<Vec<Item>> for Items {
    fn from(vec: Vec<Item>) -> Items {
        Items(vec)
    }
}

impl Deref for Items {
    type Target = Vec<Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
            if re.is_match(&item.name()) || re.is_match(&item.description()) {
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

    pub fn export(&self, filename: &'static str) -> Result<()> {
        std::fs::File::create(filename).map_err(|source| Error::FileCreate { filename, source })?;

        let mut writer = csv::Writer::from_path(filename)
            .map_err(|source| Error::FileOpen { filename, source })?;
        for item in &self.0 {
            writer
                .serialize(item)
                .map_err(|source| Error::Serialization { source })?;
        }
        writer.flush().map_err(|source| Error::Flush { source })?;
        Ok(())
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
        686
        "###);
    }
}