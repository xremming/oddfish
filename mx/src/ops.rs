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
    ///
    /// If `lhs` is falsy, then `lhs` is returned, otherwise returns `rhs`. Short-circuits.
    And,
    /// `lhs || rhs`
    ///
    /// If `lhs` is truthy, then `lhs` is returned, otherwise returns `rhs`. Short-circuits.
    Or,
    /// `lhs ?? rhs`
    ///
    /// If `lhs` is `nil`, then `rhs` is returned. Short-circuits.
    NilCoalesce,
}
