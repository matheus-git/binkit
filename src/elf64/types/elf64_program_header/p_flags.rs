use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub enum PFlagsValue {
    R,
    W,
    X,
}

impl PFlagsValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            PFlagsValue::R => "Read",
            PFlagsValue::W => "Write",
            PFlagsValue::X => "Execute",
        }
    }

    pub fn from_raw(raw: [u8; 4], endian: &Endian) -> Vec<Self> {
        let mask = endian.read_u32(raw);
        let mut flags = Vec::new();
        if mask & 0x4 != 0 { flags.push(PFlagsValue::R); }
        if mask & 0x2 != 0 { flags.push(PFlagsValue::W); }
        if mask & 0x1 != 0 { flags.push(PFlagsValue::X); }
        flags
    }
}

#[derive(Debug)]
pub struct PFlags<'a> {
    pub raw: Cow<'a, [u8; 4]>,
}

impl<'a> PFlags<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self { 
            raw
        }
    }
}

impl HeaderField for PFlags<'_> {
    type Value = Vec<PFlagsValue>;
    fn describe(&self, endian: &Endian) -> String {
        let values = self.value(endian);
        if values.is_empty() {
            "None".to_string()
        } else {
            #[allow(clippy::redundant_closure_for_method_calls)]
            values.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(" | ")
        }
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        PFlagsValue::from_raw(*self.raw, endian)
    }
}
