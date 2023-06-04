use super::Tag;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, str::FromStr};

#[derive(Clone, Debug, Deserialize, Serialize)]
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

impl FromStr for Tags {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tags: HashSet<Tag> = s
            .split(',')
            .map(|s| s.trim())
            .map(|s| Tag::from_str(s).unwrap())
            .collect();
        Ok(Tags(tags))
    }
}

impl From<&Tags> for String {
    fn from(tags: &Tags) -> Self {
        let mut tags = tags
            .0
            .iter()
            .map(|tag| tag.to_string())
            .collect::<Vec<String>>();
        tags.sort();

        tags.join(", ")
    }
}
