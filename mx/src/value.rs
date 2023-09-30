use crate::{Primitive, Table, Type, TypeOf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Primitive(Primitive),
    Table(Table),

    // TODO: FunctionNative,
    FunctionPointer(usize),
}

impl From<Table> for Value {
    fn from(value: Table) -> Self {
        Value::Table(value)
    }
}

macro_rules! into_value {
    ($t:ty = $arg:ident => $s:expr) => {
        impl From<$t> for Value {
            fn from($arg: $t) -> Self {
                Value::Primitive($s)
            }
        }
    };
    ($($t:ty),+ = $arg:ident => $s:expr) => {
        $(into_value!($t = $arg => $s);)+
    };
}

into_value!(Primitive = value => value);
into_value!(() = _value => Primitive::Nil);
into_value!(bool = value => Primitive::Bool(value));
into_value!(String = value => Primitive::String(value));
into_value!(&str = value => Primitive::String(value.to_string()));
into_value!(f64, f32 = value => Primitive::Number(value.into()));
into_value!(usize, u64, u32, u16, u8 = value => Primitive::Number(value.into()));
into_value!(isize, i64, i32, i16, i8 = value => Primitive::Number(value.into()));

macro_rules! try_from_value {
    ($t:ty) => {
        impl TryFrom<Value> for $t {
            type Error = ();

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::Primitive(value) => value.try_into(),
                    _ => Err(()),
                }
            }
        }
    };
    ($($t:ty),+) => {
        $(try_from_value!($t);)+
    };
}

try_from_value!(());
try_from_value!(bool);
try_from_value!(String);
try_from_value!(f64, f32);
try_from_value!(usize, u64, u32, u16, u8);
try_from_value!(isize, i64, i32, i16, i8);

impl Value {
    pub fn new(v: impl Into<Self>) -> Self {
        v.into()
    }

    pub fn nil() -> Self {
        Value::Primitive(Primitive::Nil)
    }

    pub fn table() -> Self {
        Table::new().into()
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Primitive(Primitive::Nil))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Primitive(Primitive::Bool(_)))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Primitive(Primitive::Number(_)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Primitive(Primitive::String(_)))
    }

    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }

    pub fn get_value<T: TryFrom<Value>>(self) -> Option<T> {
        T::try_from(self).ok()
    }

    pub fn get_primitive(self) -> Option<Primitive> {
        match self {
            Value::Primitive(value) => Some(value),
            _ => None,
        }
    }

    pub fn get_table(self) -> Option<Table> {
        match self {
            Value::Table(table) => Some(table),
            _ => None,
        }
    }
}

impl TypeOf for Value {
    fn type_of(&self) -> Type {
        match self {
            Value::Primitive(value) => value.type_of(),
            Value::Table(_) => Type::Table,
            // Value::FunctionNative => Type::Function,
            Value::FunctionPointer(_) => Type::Function,
        }
    }
}
