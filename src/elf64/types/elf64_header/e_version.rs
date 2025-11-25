use crate::traits::header_field::HeaderField;
use std::borrow::Cow;
use crate::utils::endian::Endian;
use std::fmt;

#[derive(Debug)]
pub enum EVersionValue {
    None,
    Current
}

impl EVersionValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "Invalid version",
            Self::Current => "Current version",
        }
    }
}

impl fmt::Display for EVersionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


#[derive(Debug)]
pub struct EVersion<'a> {
    pub raw: Cow<'a, [u8; 4]>,
}

impl<'a> EVersion<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for EVersion<'a> {
    type Value = EVersionValue;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).as_str().to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        match endian.read_u32(*self.raw) {
            1 => EVersionValue::Current,
            _ => EVersionValue::None
        }
    }
}
