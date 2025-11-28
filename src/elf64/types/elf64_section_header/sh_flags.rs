use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug, Clone)]
pub enum ShFlagsValue {
    A, 
    W,
    X,
    M,
}

impl ShFlagsValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "Alloc",
            Self::W => "Write",
            Self::X => "Execute",
            Self::M => "Mask",
        }
    }

    pub fn from_raw(raw: [u8; 8], endian: &Endian) -> Vec<Self> {
        let mask = endian.read_u64(raw); 
        let mut flags = Vec::new();

        if mask & 0x1 != 0 { flags.push(Self::W); }
        if mask & 0x2 != 0 { flags.push(Self::A); }
        if mask & 0x4 != 0 { flags.push(Self::X); }
        if mask & !(0x1 | 0x2 | 0x4) != 0 { flags.push(Self::M); }

        flags
    }
}

#[derive(Debug)]
pub struct ShFlags<'a> {
    pub raw: Cow<'a, [u8; 8]>,
}

impl<'a> ShFlags<'a> {
    pub fn new(raw: Cow<'a, [u8; 8]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl HeaderField for ShFlags<'_>{
    type Value = Vec<ShFlagsValue>;
    fn describe(&self, endian: &Endian) -> String {
        let values = self.value(endian);
        if values.is_empty() {
            "None".to_string()
        } else {
            values
                .iter()
                .map(ShFlagsValue::as_str)
                .collect::<Vec<_>>()
                .join(" | ")
        }
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        ShFlagsValue::from_raw(*self.raw, endian)
    }
}

