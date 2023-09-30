use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use crate::{Primitive, Value};

#[macro_export]
macro_rules! table {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut table = Table::new();
            $(
                table.set($key, $value);
            )*
            table
        }
    };
    ($($value:expr),* $(,)?) => {
        {
            let mut table = Table::new();
            let mut i = 0;
            $(
                table.set(i, $value);
                i += 1;
            )*
            table
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table(HashMap<Primitive, Value>);

impl Table {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_vec(vs: Vec<Value>) -> Self {
        let mut table = Self::new();
        for (i, v) in vs.into_iter().enumerate() {
            table.set(i, v);
        }
        table
    }

    pub fn compact(&mut self) {
        self.0.retain(|_, v| !v.is_nil());
    }
}

impl Table {
    pub fn iter_list(self) -> impl Iterator<Item = Value> {
        (0..).into_iter().map_while(move |i| match self.get(i) {
            Some(v) if v.is_nil() => None,
            Some(v) => Some(v.clone()),
            None => None,
        })
    }

    pub fn iter(self) -> impl Iterator<Item = (Primitive, Value)> {
        self.into_iter()
    }
}

impl IntoIterator for Table {
    type Item = (Primitive, Value);
    type IntoIter = std::collections::hash_map::IntoIter<Primitive, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Table {
    pub fn set(&mut self, key: impl Into<Primitive>, value: impl Into<Value>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: impl Into<Primitive>) -> Option<&Value> {
        self.0.get(&key.into())
    }

    pub fn get_mut(&mut self, key: impl Into<Primitive>) -> &mut Value {
        self.0.entry(key.into()).or_insert(Value::nil())
    }
}

impl TryFrom<Value> for Table {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Table(table) => Ok(table),
            _ => Err(()),
        }
    }
}

impl<Idx> Index<Idx> for Table
where
    Idx: Into<Primitive>,
{
    type Output = Value;

    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<Idx> IndexMut<Idx> for Table
where
    Idx: Into<Primitive>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.get_mut(index)
    }
}
