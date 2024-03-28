use crate::{Index, Value};

#[derive(Debug)]
pub enum Query<T, I: Index<T>> {
    Not(Box<Query<T, I>>),
    And(Box<Vec<Query<T, I>>>),
    Or(Box<Vec<Query<T, I>>>),
    Eq(I, Value),

    // TODO: how to get rid of this?
    _Phantom(std::marker::PhantomData<T>),
}

// pub struct Query {}

impl<T, I: Index<T>> Query<T, I> {
    pub fn and(children: impl IntoIterator<Item = Query<T, I>>) -> Query<T, I> {
        Query::And(children.into_iter().collect::<Vec<_>>().into())
    }

    pub fn or(children: impl IntoIterator<Item = Query<T, I>>) -> Query<T, I> {
        Query::Or(children.into_iter().collect::<Vec<_>>().into())
    }

    pub fn eq(lhs: I, rhs: Value) -> Query<T, I> {
        Query::Eq(lhs, rhs)
    }
}
