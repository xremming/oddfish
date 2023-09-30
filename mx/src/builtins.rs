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

pub(crate) fn bool(value: impl Into<Value>) -> bool {
    use Primitive::*;

    match value.into() {
        Value::Primitive(value) => match value {
            Nil => false,
            Bool(value) => value,
            Number(value) => {
                if value.is_nan() {
                    true
                } else {
                    *value != 0.0
                }
            }
            String(value) => !value.is_empty(),
        },
        Value::Table(table) => table.into_iter().any(|_| true),
        // Value::FunctionNative => true,
        Value::FunctionPointer(_) => true,
    }
}

#[cfg(test)]
mod test {
    use crate::table;

    use super::*;

    #[test]
    fn test_bool_nil() {
        assert_eq!(bool(()), false);
    }

    #[test]
    fn test_bool_bool() {
        assert_eq!(bool(true), true);
        assert_eq!(bool(false), false);
    }

    #[test]
    fn test_bool_number() {
        assert_eq!(bool(0), false);
        assert_eq!(bool(0.0), false);
        assert_eq!(bool(-0.0), false);
        assert_eq!(bool(1), true);
        assert_eq!(bool(1.0), true);
        assert_eq!(bool(-1.0), true);
        assert_eq!(bool(0.1), true);
        assert_eq!(bool(-0.1), true);
        assert_eq!(bool(f64::INFINITY), true);
        assert_eq!(bool(-f64::INFINITY), true);
        assert_eq!(bool(f64::NAN), true);
        assert_eq!(bool(-f64::NAN), true);
    }

    #[test]
    fn test_bool_table() {
        assert_eq!(bool(table![]), false);
        assert_eq!(bool(table![()]), false);
        assert_eq!(bool(table![1]), true);
        assert_eq!(bool(table!["a" => 1]), true);
        assert_eq!(bool(table!["a" => ()]), false);
        assert_eq!(bool(table!["a" => 1, "b" => ()]), true);
    }
}
