use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
    ops::Bound,
};

use crate::{ItemID, Value};

pub trait IndexStorage: Debug {
    fn add(&mut self, item_id: ItemID, value: Value) -> bool;
    fn remove(&mut self, item_id: ItemID, value: Value) -> bool;
    fn get(&self, value: &Value) -> Vec<ItemID>;

    fn update(&mut self, item_id: ItemID, old_value: Value, new_value: Value) {
        self.remove(item_id, old_value);
        self.add(item_id, new_value);
    }
}

#[derive(Debug, Default)]
pub struct NonUniqueIndexStorage(BTreeMap<(Value, ItemID), ()>);

impl IndexStorage for NonUniqueIndexStorage {
    fn add(&mut self, item_id: ItemID, value: Value) -> bool {
        self.0.insert((value, item_id), ());
        true
    }

    fn get(&self, value: &Value) -> Vec<ItemID> {
        let mut cursor = self
            .0
            .lower_bound(Bound::Included(&(value.clone(), ItemID::new(0))));

        let mut out = Vec::new();
        while let Some(((next_value, next_item_id), _)) = cursor.next() {
            if next_value != value {
                break;
            }

            out.push(*next_item_id);
        }

        out
    }

    fn remove(&mut self, item_id: ItemID, value: Value) -> bool {
        self.0.remove(&(value, item_id)).is_some()
    }
}

#[derive(Debug, Default)]
pub struct UniqueIndexStorage(BTreeMap<Value, ItemID>);

impl IndexStorage for UniqueIndexStorage {
    fn add(&mut self, item_id: ItemID, value: Value) -> bool {
        match self.0.entry(value) {
            Entry::Vacant(e) => {
                e.insert(item_id);
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    fn get(&self, value: &Value) -> Vec<ItemID> {
        match self.0.get(value) {
            Some(item_id) => vec![*item_id],
            None => vec![],
        }
    }

    fn remove(&mut self, item_id: ItemID, value: Value) -> bool {
        match self.0.remove(&value) {
            Some(old_item_id) => {
                assert_eq!(item_id, old_item_id);
                true
            }
            None => false,
        }
    }
}

pub fn new_index_storage(unique: bool) -> Box<dyn IndexStorage> {
    if unique {
        Box::new(UniqueIndexStorage::default()) as Box<dyn IndexStorage>
    } else {
        Box::new(NonUniqueIndexStorage::default()) as Box<dyn IndexStorage>
    }
}
