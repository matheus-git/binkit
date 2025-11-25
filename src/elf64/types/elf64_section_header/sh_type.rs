use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug, Clone)]
pub enum ShTypeValue {
    Null,
    ProgBits,
    SymTab,
    StrTab,
    Rela,
    Hash,
    Dynamic,
    Note,
    NoBits,
    Rel,
    DynSym,
    Other(()), 
}

impl ShTypeValue {
    pub fn from_u32(raw: u32) -> Self {
        match raw {
            0 => ShTypeValue::Null,
            1 => ShTypeValue::ProgBits,
            2 => ShTypeValue::SymTab,
            3 => ShTypeValue::StrTab,
            4 => ShTypeValue::Rela,
            5 => ShTypeValue::Hash,
            6 => ShTypeValue::Dynamic,
            7 => ShTypeValue::Note,
            8 => ShTypeValue::NoBits,
            9 => ShTypeValue::Rel,
            11 => ShTypeValue::DynSym,
            _ => ShTypeValue::Other(()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ShTypeValue::Null => "NULL",
            ShTypeValue::ProgBits => "PROGBITS",
            ShTypeValue::SymTab => "SYMTAB",
            ShTypeValue::StrTab => "STRTAB",
            ShTypeValue::Rela => "RELA",
            ShTypeValue::Hash => "HASH",
            ShTypeValue::Dynamic => "DYNAMIC",
            ShTypeValue::Note => "NOTE",
            ShTypeValue::NoBits => "NOBITS",
            ShTypeValue::Rel => "REL",
            ShTypeValue::DynSym => "DYNSYM",
            ShTypeValue::Other(_) => "OTHER",
        }
    }
}

#[derive(Debug)]
pub struct ShType<'a> {
    pub raw: Cow<'a, [u8; 4]>,
}

impl<'a> ShType<'a> {
    pub fn new(raw: Cow<'a, [u8; 4]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for ShType<'a> {
    type Value = ShTypeValue;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).as_str().to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
        ShTypeValue::from_u32(endian.read_u32(*self.raw))
    }
}
