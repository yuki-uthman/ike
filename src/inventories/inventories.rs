use super::inventory::Inventory;
use crate::loader::Loader;
use std::ops::Deref;

#[derive(Debug)]
pub struct Inventories(Vec<Inventory>);

impl Loader<Inventory> for Inventories {}
impl From<Vec<Inventory>> for Inventories {
    fn from(vec: Vec<Inventory>) -> Inventories {
        Inventories(vec)
    }
}

// https://stackoverflow.com/questions/68277992/implement-iterator-trait-for-a-struct-containing-an-iterable-field
impl Deref for Inventories {
    type Target = Vec<Inventory>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Inventories {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;

    #[test]
    fn len() {
        let inventories = Inventories::load("assets/Inventory.csv").unwrap();
        assert_yaml_snapshot!(inventories.len(), @r###"
        ---
        3
        "###);
    }
}
