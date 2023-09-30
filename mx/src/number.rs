use std::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    str::FromStr,
};

/// Wraps f64 in such a way that it supports full ordering, hashing, and equality.
///
/// NaN is treated as less than all other values, and equal to itself.
#[derive(Clone, Copy)]
pub struct Number(f64);

impl Number {
    pub fn new(n: f64) -> Self {
        Self(n)
    }

    /// Parse string into a number, on failure silently returns NaN.
    pub fn parse(s: &str) -> Number {
        match f64::from_str(s) {
            Ok(v) => Number(v),
            Err(_) => Number(f64::NAN),
        }
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq<Number> for Number {
    fn eq(&self, other: &Number) -> bool {
        if self.0.is_nan() && other.0.is_nan() {
            return true;
        }

        self.0 == other.0
    }
}

impl Eq for Number {}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.is_nan() {
            f64::MIN.to_bits().hash(state);
        } else {
            self.0.to_bits().hash(state);
        }
    }
}

impl PartialOrd<Number> for Number {
    fn partial_cmp(&self, other: &Number) -> Option<Ordering> {
        if self.0.is_nan() && other.0.is_nan() {
            return Some(Ordering::Equal);
        }

        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Number) -> Ordering {
        let self_is_nan = self.0.is_nan();
        let other_is_nan = other.0.is_nan();

        match (self_is_nan, other_is_nan) {
            (true, true) => return Ordering::Equal,
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

macro_rules! impl_from_for_number {
    ($t:ty) => {
        impl From<$t> for Number {
            fn from(n: $t) -> Self {
                Self(n as f64)
            }
        }

        impl From<Number> for $t {
            fn from(n: Number) -> Self {
                n.0 as $t
            }
        }
    };
    ($($t:ty),*) => {
        $(impl_from_for_number!($t);)*
    };
}

impl_from_for_number!(f64, f32);
impl_from_for_number!(usize, u128, u64, u32, u16, u8);
impl_from_for_number!(isize, i128, i64, i32, i16, i8);

impl Deref for Number {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Number {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
