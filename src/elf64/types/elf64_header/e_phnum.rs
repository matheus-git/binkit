use std::borrow::Cow;

use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct EPhnum<'a> {
    pub raw: Cow<'a, [u8; 2]>,
}

impl<'a> EPhnum<'a> {
    pub fn new(raw: Cow<'a, [u8; 2]>) -> Self {
        Self { 
            raw 
        }
    }
}

impl<'a> HeaderField for EPhnum<'a> {
    type Value = u16;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        endian.read_u16(*self.raw)
    }
}
