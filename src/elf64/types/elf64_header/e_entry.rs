use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct EEntry<'a> {
    pub raw: Cow<'a, [u8; 8]>,
}

impl<'a> EEntry<'a> {
    pub fn new(raw: Cow<'a, [u8; 8]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for EEntry<'a> {
    type Value = String;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        format!("0x{:X}", endian.read_u64(*self.raw))
    }
}
