use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    Blob,
    String,
    Float,
    Int,
    Bool,
}

#[derive(Debug, Clone)]
pub enum Value {
    Blob(Vec<u8>),
    String(String),
    Float(f64),
    Int(i64),
    Bool(bool),
}

impl Value {
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Blob(_) => DataType::Blob,
            Value::String(_) => DataType::String,
            Value::Float(_) => DataType::Float,
            Value::Int(_) => DataType::Int,
            Value::Bool(_) => DataType::Bool,
        }
    }

    pub fn blob(data: impl IntoIterator<Item = u8>) -> Self {
        Value::Blob(data.into_iter().collect())
    }

    pub fn string(data: impl ToString) -> Self {
        Value::String(data.to_string())
    }

    pub fn float(data: f64) -> Self {
        Value::Float(data.into())
    }

    pub fn int(data: i64) -> Self {
        Value::Int(data.into())
    }

    pub fn bool(data: impl Into<bool>) -> Self {
        Value::Bool(data.into())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Blob(a), Value::Blob(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => {
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Blob(a), Value::Blob(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => match (a.is_nan(), b.is_nan()) {
                (true, true) => Some(Ordering::Equal),
                (true, false) => Some(Ordering::Less),
                (false, true) => Some(Ordering::Greater),
                (false, false) => a.partial_cmp(b),
            },
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
            (a, b) => a.data_type().partial_cmp(&b.data_type()),
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
