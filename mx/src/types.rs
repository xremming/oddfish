#[derive(Debug)]
pub enum Type {
    Nil,
    Bool,
    Number,
    String,
    Table,
    Function,
}

pub trait TypeOf {
    fn type_of(&self) -> Type;
}
