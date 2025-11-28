use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub enum PTypeValue {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Loproc,
    GnuStack,
    Unknown(()),
}

impl PTypeValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            PTypeValue::Null => "NULL",
            PTypeValue::Load => "LOAD",
            PTypeValue::Dynamic => "DYNAMIC",
            PTypeValue::Interp => "INTERP",
            PTypeValue::Note => "NOTE",
            PTypeValue::Shlib => "SHLIB",
            PTypeValue::Phdr => "PHDR",
            PTypeValue::Loproc => "LOPROC",
            PTypeValue::GnuStack => "GNU_STACK",
            PTypeValue::Unknown(()) => "UNKNOWN",
        }
    }

    pub fn from_raw(raw: [u8; 4], endian: &Endian) -> Self {
        const PT_LOPROC: u32 = 0x7000_0000;
        const PT_HIPROC: u32 = 0x7fff_ffff;
        const PT_GNU_STACK: u32 = 0x6474_e550;

        let val = endian.read_u32(raw);

        match val {
            0 => PTypeValue::Null,
            1 => PTypeValue::Load,
            2 => PTypeValue::Dynamic,
            3 => PTypeValue::Interp,
            4 => PTypeValue::Note,
            5 => PTypeValue::Shlib,
            6 => PTypeValue::Phdr,
            PT_LOPROC..=PT_HIPROC => PTypeValue::Loproc,
            PT_GNU_STACK => PTypeValue::GnuStack,
            _ => PTypeValue::Unknown(()),
        }
    }
}

#[derive(Debug)]
pub struct PType<'a> {
    pub raw: Cow<'a, [u8; 4]>,
}

impl<'a> PType<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self {
            raw,
        }
    }
}

impl HeaderField for PType<'_> {
    type Value = PTypeValue;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).as_str().to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        PTypeValue::from_raw(*self.raw, endian)
    }
}
