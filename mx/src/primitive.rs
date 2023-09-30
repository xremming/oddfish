use crate::{Number, Type, TypeOf, Value};

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Primitive {
    #[default]
    Nil,
    Bool(bool),
    Number(Number),
    String(String),
}

impl Primitive {
    fn is_nil(&self) -> bool {
        matches!(self, Primitive::Nil)
    }

    fn is_bool(&self) -> bool {
        matches!(self, Primitive::Bool(_))
    }

    fn is_number(&self) -> bool {
        matches!(self, Primitive::Number(_))
    }

    fn is_string(&self) -> bool {
        matches!(self, Primitive::String(_))
    }
}

impl TryFrom<Value> for Primitive {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Primitive(value) => Ok(value),
            _ => Err(()),
        }
    }
}

impl From<()> for Primitive {
    fn from(_: ()) -> Self {
        Primitive::Nil
    }
}

impl TryFrom<Primitive> for () {
    type Error = ();

    fn try_from(value: Primitive) -> Result<Self, Self::Error> {
        match value {
            Primitive::Nil => Ok(()),
            _ => Err(()),
        }
    }
}

impl From<bool> for Primitive {
    fn from(value: bool) -> Self {
        Primitive::Bool(value)
    }
}

impl TryFrom<Primitive> for bool {
    type Error = ();

    fn try_from(value: Primitive) -> Result<Self, Self::Error> {
        match value {
            Primitive::Bool(value) => Ok(value),
            _ => Err(()),
        }
    }
}

macro_rules! from_number {
    ($t:ty) => {
        impl From<$t> for Primitive {
            fn from(value: $t) -> Self {
                Primitive::Number(value.into())
            }
        }

        impl TryFrom<Primitive> for $t {
            type Error = ();

            fn try_from(value: Primitive) -> Result<Self, Self::Error> {
                match value {
                    Primitive::Number(value) => Ok(value.into()),
                    _ => Err(()),
                }
            }
        }
    };

    ($t:ty, $($rest:ty),+) => {
        from_number!($t);
        from_number!($($rest),+);
    };
}

from_number!(f64, f32);
from_number!(isize, i64, i32, i16, i8);
from_number!(usize, u64, u32, u16, u8);

impl From<&str> for Primitive {
    fn from(value: &str) -> Self {
        Primitive::String(value.to_string())
    }
}

impl From<String> for Primitive {
    fn from(value: String) -> Self {
        Primitive::String(value)
    }
}

impl TryFrom<Primitive> for String {
    type Error = ();

    fn try_from(value: Primitive) -> Result<Self, Self::Error> {
        match value {
            Primitive::String(value) => Ok(value),
            _ => Err(()),
        }
    }
}

impl TypeOf for Primitive {
    fn type_of(&self) -> Type {
        use Primitive::*;
        match self {
            Nil => Type::Nil,
            Bool(_) => Type::Bool,
            Number(_) => Type::Number,
            String(_) => Type::String,
        }
    }
}
