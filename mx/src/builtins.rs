use crate::{Primitive, Value};

pub(crate) fn str(value: Value) -> String {
    use Primitive::*;

    match value {
        Value::Primitive(value) => match value {
            Nil => "nil".to_string(),
            Bool(value) => value.to_string(),
            Number(value) => value.to_string(),
            String(value) => value,
        },
        // TODO: print table contents
        // TODO: use __str method if it exists
        Value::Table(_) => "{table}".to_string(),
        // Value::FunctionNative => "<native function>".to_string(),
        Value::FunctionPointer(_) => "<function>".to_string(),
    }
}

pub(crate) fn bool(value: Value) -> bool {
    use Primitive::*;

    match value {
        Value::Primitive(value) => match value {
            Nil => false,
            Bool(value) => value,
            Number(value) => {
                if value.is_nan() {
                    false
                } else {
                    *value != 0.0
                }
            }
            String(value) => !value.is_empty(),
        },
        Value::Table(table) => table.into_iter().any(|(_, v)| !v.is_nil()),
        // Value::FunctionNative => true,
        Value::FunctionPointer(_) => true,
    }
}
