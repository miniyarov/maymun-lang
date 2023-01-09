use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
    Return(Box<Object>),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Null => write!(f, "null"),
            Object::Return(o) => write!(f, "{}", o),
        }
    }
}
