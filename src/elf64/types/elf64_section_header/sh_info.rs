use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct ShInfo<'a> {
    pub raw: Cow<'a, [u8; 4]>,
}

impl<'a> ShInfo<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl HeaderField for ShInfo<'_> {
    type Value = u32;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        endian.read_u32(*self.raw)
    }
}
