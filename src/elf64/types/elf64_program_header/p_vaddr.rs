use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct PVaddr<'a> {
    pub raw: Cow<'a, [u8; 8]>,
}

impl<'a> PVaddr<'a> {
    pub fn new(raw: Cow<'a, [u8; 8]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for PVaddr<'a> {
    type Value = String;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).clone()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        format!("0x{:X}", endian.read_u64(*self.raw))
    }
}
