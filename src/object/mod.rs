use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    Null,
    Return(Box<Object>),
    Error(String),
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "Integer({})", i),
            Object::Boolean(b) => write!(f, "Boolean({})", b),
            Object::Null => write!(f, "Null"),
            Object::Return(o) => write!(f, "Return({})", o),
            Object::Error(msg) => write!(f, "Error({})", msg),
        }
    }
}

pub type Environment = HashMap<String, Object>;
