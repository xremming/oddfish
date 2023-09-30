pub(crate) enum UnaryOp {
    /// `+val`
    Plus,
    /// `-val`
    Minus,
    /// `!val`
    Not,
}

pub(crate) enum BinaryOp {
    /// `lhs + rhs`
    Add,
    /// `lhs - rhs`
    Sub,
    /// `lhs * rhs`
    Mul,
    /// `lhs / rhs`
    Div,
    /// `lhs % rhs`
    Mod,
    /// `lhs // rhs`
    IntegerDiv,
    /// `lhs ^ rhs`
    Pow,
    /// `lhs == rhs`
    Eq,
    /// `lhs != rhs`
    Ne,
    /// `lhs < rhs`
    Lt,
    /// `lhs <= rhs`
    Lte,
    /// `lhs > rhs`
    Gt,
    /// `lhs >= rhs`
    Gte,
    /// `lhs && rhs`
    And,
    /// `lhs || rhs`
    Or,
}
