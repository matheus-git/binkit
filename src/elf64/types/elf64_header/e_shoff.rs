use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct EShoff<'a> {
    pub raw: Cow<'a, [u8; 8]>,
}

impl<'a> EShoff<'a> {
    pub fn new(raw: Cow<'a, [u8; 8]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for EShoff<'a> {
    type Value = u64;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        endian.read_u64(*self.raw)
    }
}
