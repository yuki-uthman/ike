use std::ops::Deref;

use crate::Items;
use crate::Tag;

pub struct Group<'a> {
    pub should_print: bool,
    pub regex: String,
    tags: &'a [Tag],
    items: Option<Items>,
}

impl<'a> Group<'a> {
    pub fn new<S>(regex: S, tags: &'a [Tag], should_print: bool) -> Self
    where
        S: Into<String>,
    {
        Self {
            regex: regex.into(),
            tags,
            items: None,
            should_print,
        }
    }

    pub fn add_items(&mut self, mut items: Items) {
        for item in items.iter_mut() {
            item.add_tags(self.tags);
        }
        self.items = Some(items);
    }

    pub fn print(&self) {
        println!("{}", self.regex);
        for item in self.items.as_ref().unwrap().iter() {
            println!("  {}", item.name());
        }
        println!();
    }
}

impl Deref for Group<'_> {
    type Target = Items;

    fn deref(&self) -> &Self::Target {
        self.items.as_ref().unwrap()
    }
}
