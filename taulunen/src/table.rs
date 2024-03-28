use crate::{new_index_storage, DataType, IndexStorage, ItemID, ItemIDGenerator, Value};

use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

pub trait Index<T>: Eq + Hash {
    fn data_type(&self) -> DataType;
    fn extract(&self, item: &T) -> Option<Value>;
    fn is_unique(&self) -> bool;

    fn is_nullable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Table<T: Clone, I: Index<T>> {
    item_id: ItemIDGenerator,
    items: HashMap<ItemID, T>,
    indices: HashMap<I, Box<dyn IndexStorage>>,
}

impl<T: Clone, I: Index<T>> Default for Table<T, I> {
    fn default() -> Self {
        Table {
            item_id: ItemIDGenerator::default(),
            items: HashMap::new(),
            indices: HashMap::new(),
        }
    }
}

impl<T: Clone, I: Index<T>> Table<T, I> {
    #[must_use]
    pub fn empty() -> Self {
        Table::default()
    }

    #[must_use]
    pub fn add_index(mut self, index: I) -> Self {
        let unique = index.is_unique();
        match self.indices.entry(index) {
            Entry::Occupied(_) => return self,
            Entry::Vacant(e) => e.insert(new_index_storage(unique)),
        };

        if self.items.len() == 0 {
            return self;
        }

        todo!("Build the index");
    }

    #[must_use]
    pub fn with_indices(indices: impl IntoIterator<Item = I>) -> Self {
        let mut table = Table::default();
        for index in indices {
            table = table.add_index(index);
        }

        table
    }
}

impl<T: Clone, I: Index<T>> Table<T, I> {
    fn index_item(&mut self, item_id: ItemID, item: &T) {
        for (index, index_storage) in self.indices.iter_mut() {
            match index.extract(&item) {
                Some(index_value) => {
                    let index_data_type = index.data_type();
                    if index_value.data_type() != index_data_type {
                        todo!("Return an Err instead of panicking");
                    }

                    index_storage.add(item_id, index_value);
                }
                None => (),
            };
        }
    }

    fn unindex_item(&mut self, item_id: ItemID, item: &T) {
        for (index, index_storage) in self.indices.iter_mut() {
            match index.extract(&item) {
                Some(index_value) => {
                    let index_data_type = index.data_type();
                    if index_value.data_type() != index_data_type {
                        todo!("Return an Err instead of panicking");
                    }

                    index_storage.remove(item_id, index_value);
                }
                None => (),
            };
        }
    }

    fn reindex_item(&mut self, item_id: ItemID, old_item: &T, new_item: &T) {
        for (index, index_storage) in self.indices.iter_mut() {
            match (index.extract(&old_item), index.extract(&new_item)) {
                (Some(old_index_value), Some(new_index_value)) => {
                    if old_index_value == new_index_value {
                        continue;
                    } else if old_index_value.data_type() != new_index_value.data_type() {
                        todo!("Return an Err instead of panicking");
                    } else if old_index_value.data_type() != index.data_type() {
                        todo!("Return an Err instead of panicking");
                    }

                    index_storage.update(item_id, old_index_value, new_index_value);
                }
                _ => (),
            };
        }
    }
}

impl<T: Clone, I: Index<T>> Table<T, I> {
    pub fn insert(&mut self, item: T) -> ItemID {
        let item_id = self.item_id.next();
        self.index_item(item_id, &item);
        self.items.insert(item_id, item);

        item_id
    }

    pub fn get(&self, item_id: ItemID) -> Option<T> {
        self.items.get(&item_id).cloned()
    }

    pub fn update<O>(&mut self, item_id: ItemID, update: impl FnOnce(&mut T) -> O) -> Option<O> {
        if let Some((old_item, new_item, out)) = match self.items.get_mut(&item_id) {
            Some(item) => {
                let old_item = item.clone();
                let out = update(item);
                Some((old_item, item.clone(), out))
            }
            None => None,
        } {
            self.reindex_item(item_id, &old_item, &new_item);
            Some(out)
        } else {
            None
        }
    }

    /// Removes the item with [`item_id`](ItemID) from the [`Table`], returning
    /// the removed item.
    ///
    /// Will not vaccuum indices automatically potentially leaving "dangling"
    /// ItemIDs there.
    pub fn remove(&mut self, item_id: ItemID) -> Option<T> {
        if let Some(out) = self.items.remove(&item_id) {
            self.unindex_item(item_id, &out);
            Some(out)
        } else {
            None
        }
    }

    pub fn remove_if(&mut self, item_id: ItemID, remove_if: impl FnOnce(&T) -> bool) -> Option<T> {
        match self.items.entry(item_id) {
            Entry::Occupied(e) => {
                if remove_if(e.get()) {
                    let item = e.remove();
                    self.unindex_item(item_id, &item);
                    Some(item)
                } else {
                    None
                }
            }
            Entry::Vacant(_) => None,
        }
    }
}

impl<T: Clone, I: Index<T>> Table<T, I> {
    pub fn where_eq(&self, index: I, value: Value) -> Vec<T> {
        let item_ids = match self.indices.get(&index) {
            Some(index_storage) => index_storage.get(&value),
            None => vec![],
        };

        let mut out = Vec::with_capacity(item_ids.len());
        for item_id in item_ids {
            if let Some(item) = self.get(item_id) {
                out.push(item);
            }
        }

        out
    }
}
