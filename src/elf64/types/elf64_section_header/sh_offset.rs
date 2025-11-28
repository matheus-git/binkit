use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct ShOffset<'a> {
    pub raw: Cow<'a, [u8; 8]>,
}

impl<'a> ShOffset<'a> {
    pub fn new(raw: Cow<'a, [u8; 8]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl HeaderField for ShOffset<'_> {
    type Value = u64;
    fn describe(&self, endian: &Endian) -> String {
        format!("0x{:X}", endian.read_u64(*self.raw))
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        endian.read_u64(*self.raw)
    }
}
