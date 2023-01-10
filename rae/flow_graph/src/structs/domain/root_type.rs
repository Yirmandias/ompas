use crate::structs::domain::root_type::RootType::*;
use crate::structs::domain::TypeId;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum RootType {
    Empty = 0,
    Any = 1,
    Boolean = 2,
    List = 3,
    Map = 4,
    Err = 5,
    Handle = 6,
    Number = 7,
    Nil = 8,
    Int = 9,
    Float = 10,
    Symbol = 11,
    EmptyList = 12,
    True = 13,
    False = 14,
}

pub const TRUE_ID: usize = 13;
pub const FALSE_ID: usize = 14;
pub const NIL_ID: usize = 8;

impl TryFrom<TypeId> for RootType {
    type Error = ();

    fn try_from(value: TypeId) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Empty,
            1 => Any,
            2 => Boolean,
            3 => List,
            4 => Map,
            5 => Err,
            6 => Handle,
            7 => Number,
            8 => Nil,
            9 => Int,
            10 => Float,
            11 => Symbol,
            12 => EmptyList,
            13 => True,
            14 => False,
            _ => return Result::Err(()),
        })
    }
}

impl Display for RootType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Any => write!(f, "Any"),
            Boolean => write!(f, "Boolean"),
            List => write!(f, "List"),
            Empty => write!(f, "Empty"),
            Map => write!(f, "Map"),
            Err => write!(f, "Err"),
            Handle => write!(f, "Handle"),
            Number => write!(f, "Number"),
            Int => write!(f, "Int"),
            Float => write!(f, "Float"),
            Symbol => write!(f, "Symbol"),
            EmptyList => write!(f, "EmptyList"),
            True => write!(f, "True"),
            False => write!(f, "False"),
            Nil => write!(f, "Nil"),
        }
    }
}
