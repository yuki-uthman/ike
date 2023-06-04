use std::collections::HashSet;
use super::Tag;

pub struct Tags(HashSet<Tag>);

impl Tags {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn insert(&mut self, tag: Tag) {
        self.0.insert(tag);
    }

    pub fn contains(&self, tag: &Tag) -> bool {
        self.0.contains(tag)
    }
}
