use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub struct ShName<'a> {
    pub raw: Cow<'a, [u8; 4]>,
    pub name: Cow<'a , String>
}

impl<'a> ShName<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self { 
            raw, 
            name: Cow::Owned(String::new())
        }
    }

    pub fn update_name(&mut self, name: String){
        self.name = Cow::Owned(name);
    }
}

impl<'a> HeaderField for ShName<'a> {
    type Value = u32;
    fn describe(&self, _endian: &Endian) -> String {
        self.name.to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        endian.read_u32(*self.raw)
    }
} 
