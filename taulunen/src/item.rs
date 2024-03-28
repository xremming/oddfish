use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemID(u64);

impl ItemID {
    pub fn new(value: u64) -> ItemID {
        ItemID(value)
    }
}

#[derive(Debug, Default)]
pub struct ItemIDGenerator(AtomicU64);

impl ItemIDGenerator {
    pub fn new(first_value: u64) -> ItemIDGenerator {
        ItemIDGenerator(AtomicU64::new(first_value))
    }

    pub fn next(&mut self) -> ItemID {
        ItemID(self.0.fetch_add(1, Ordering::SeqCst))
    }
}
