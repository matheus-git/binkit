use std::fmt;
use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};

#[derive(Debug)]
pub enum EMachineValue {
    None,
    M32,
    Sparc,
    EM386,
    EM68K,
    EM88K,
    EM860,
    Mips,
    Parisc,
    Sparc32Plus,
    Ppc,
    Ppc64,
    S390,
    Arm,
    Sh,
    SparcV9,
    Ia64,
    X86_64,
    Vax,
}

impl EMachineValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            EMachineValue::None => "An unknown machine",
            EMachineValue::M32 => "AT&T WE 32100",
            EMachineValue::Sparc => "Sun Microsystems SPARC",
            EMachineValue::EM386 => "Intel 80386",
            EMachineValue::EM68K => "Motorola 68000",
            EMachineValue::EM88K => "Motorola 88000",
            EMachineValue::EM860 => "Intel 80860",
            EMachineValue::Mips => "MIPS RS3000 (big-endian only)",
            EMachineValue::Parisc => "HP/PA",
            EMachineValue::Sparc32Plus => "SPARC with enhanced instruction set",
            EMachineValue::Ppc => "PowerPC 32-bit",
            EMachineValue::Ppc64 => "PowerPC 64-bit",
            EMachineValue::S390 => "IBM S390",
            EMachineValue::Arm => "ARM",
            EMachineValue::Sh => "Renesas SuperH",
            EMachineValue::SparcV9 => "SPARC v9 64-bit",
            EMachineValue::Ia64 => "Intel Itanium",
            EMachineValue::X86_64 => "x86-64",
            EMachineValue::Vax => "DEC VAX",
        }
    }
}

impl fmt::Display for EMachineValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug)]
pub struct EMachine<'a> {
    pub raw: Cow<'a, [u8; 2]>,
}

impl<'a> EMachine<'a> {
    pub fn new(raw: Cow<'a, [u8; 2]>) -> Self {
        Self { 
            raw, 
        }
    }
}

impl<'a> HeaderField for EMachine<'a> {
    type Value = EMachineValue;
    fn describe(&self, endian: &Endian) -> String {
        self.value(endian).as_str().to_string()
    }
    fn value(&self, endian: &Endian) -> Self::Value {
       match endian.read_u16(*self.raw) {
            0 => EMachineValue::None,
            1 => EMachineValue::M32,
            2 => EMachineValue::Sparc,
            3 => EMachineValue::EM386,
            4 => EMachineValue::EM68K,
            5 => EMachineValue::EM88K,
            7 => EMachineValue::EM860,
            8 => EMachineValue::Mips,
            15 => EMachineValue::Parisc,
            18 => EMachineValue::Sparc32Plus,
            20 => EMachineValue::Ppc,
            21 => EMachineValue::Ppc64,
            22 => EMachineValue::S390,
            40 => EMachineValue::Arm,
            42 => EMachineValue::Sh,
            43 => EMachineValue::SparcV9,
            50 => EMachineValue::Ia64,
            62 => EMachineValue::X86_64,
            75 => EMachineValue::Vax,
            _ => EMachineValue::None,
        }    
    }
}

impl<'a> From<&EMachine<'a>> for Vec<u8> {
    fn from(h: &EMachine) -> Vec<u8> {
        h.raw.to_vec()
    }
}

