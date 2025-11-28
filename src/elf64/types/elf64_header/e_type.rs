use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};
use std::fmt;

#[derive(Debug)]
pub enum ETypeValue {
    None,
    Rel,
    Exec,
    Dyn,
    Core
}

impl ETypeValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "An unknown type",
            Self::Rel => "A relocatable file",
            Self::Exec => "An executable file",
            Self::Dyn => "A shared object",
            Self::Core => "A core file"
        }
    }
}

impl fmt::Display for ETypeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct EType<'a> {
    pub raw: Cow<'a, [u8; 2]>,
}

impl<'a> EType<'a> {
    pub fn new(raw: Cow<'a, [u8; 2]>) -> Self {
        Self { 
            raw
        }
    }
}

impl HeaderField for EType<'_> {
    type Value = ETypeValue;
    fn describe(&self,endian: &Endian) -> String {
        self.value(endian).as_str().to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        match endian.read_u16(*self.raw) {
            1 => ETypeValue::Rel,
            2 => ETypeValue::Exec,
            3 => ETypeValue::Dyn,
            4 => ETypeValue::Core,
            _ => ETypeValue::None
        }
    }
}

impl From<&EType<'_>> for Vec<u8> {
    fn from(h: &EType) -> Vec<u8> {
        h.raw.to_vec()
    }
}
