use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct ShLink<'a> {
    pub raw: Cow<'a, [u8; 4]>,
}

impl<'a> ShLink<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for ShLink<'a> {
    type Value = u32;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        endian.read_u32(*self.raw)
    }
}
