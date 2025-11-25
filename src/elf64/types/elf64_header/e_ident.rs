use std::borrow::Cow;
use crate::{traits::header_field::HeaderField, utils::endian::Endian};
use crate::utils::bytes_to_hex::bytes_to_hex;

#[derive(Debug)]
enum EiClass {
    Class32,
    Class64,
    ClassNone
}

impl EiClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Class32 => "Elf32",
            Self::Class64 => "Elf64",
            _ => "None"
        }
    }
}

#[derive(Debug)]
enum EiData {
    DataNone,
    Data2LSB, 
    Data2MSB  
}

impl EiData {
    pub fn as_str(&self) -> &'static str {
        match self {
            EiData::DataNone => "None",
            EiData::Data2LSB => "Little-endian (2's complement)",
            EiData::Data2MSB => "Big-endian (2's complement)",
        }
    }
}

#[derive(Debug)]
enum EiVersion {
    None,
    Current
}

#[derive(Debug)]
enum EiOsabi {
    None,
    Sysv,
    Hpux,
    Netbsd,
    Linux,
    Solaris,
    Irix,
    FreeBsd,
    Tru64,
    Arm,
    Standalone,
}

impl EiOsabi {
    pub fn as_str(&self) -> &'static str {
        match self {
            EiOsabi::None => "None",
            EiOsabi::Sysv => "UNIX System V",
            EiOsabi::Hpux => "HP-UX",
            EiOsabi::Netbsd => "NetBSD",
            EiOsabi::Linux => "Linux",
            EiOsabi::Solaris => "Solaris",
            EiOsabi::Irix => "IRIX",
            EiOsabi::FreeBsd => "FreeBSD",
            EiOsabi::Tru64 => "Tru64 UNIX",
            EiOsabi::Arm => "ARM",
            EiOsabi::Standalone => "Standalone Application",
        }
    }
}

#[derive(Debug)]
pub struct EIdent<'a> {
    pub raw: Cow<'a, [u8; 16]>,
}

impl<'a> EIdent<'a> {
    pub fn new(raw: Cow<'a, [u8; 16]>) -> Self {
        Self { 
            raw, 
        }
    }

    fn ei_class(&self) -> EiClass {
        match self.raw[4] {
            1 => EiClass::Class32,
            2 => EiClass::Class64,
            _ => EiClass::ClassNone,
        }
    }

    fn ei_data(&self) -> EiData {
        match self.raw[5] {
            1 => EiData::Data2LSB,
            2 => EiData::Data2MSB,
            _ => EiData::DataNone,
        }
    }

    pub fn endian(&self) -> Endian {
        match &self.ei_data() {
            EiData::DataNone => Endian::Little,
            EiData::Data2LSB => Endian::Little,
            EiData::Data2MSB => Endian::Big
        }
    }

    fn ei_version(&self) -> EiVersion {
        match self.raw[6] {
            1 => EiVersion::Current,
            _ => EiVersion::None,
        }
    }

    fn ei_osabi(&self) -> EiOsabi {
        match self.raw[7] {
            0 => EiOsabi::Sysv,
            1 => EiOsabi::Hpux,
            2 => EiOsabi::Netbsd,
            3 => EiOsabi::Linux,
            6 => EiOsabi::Solaris,
            8 => EiOsabi::Irix,
            9 => EiOsabi::FreeBsd,
            10 => EiOsabi::Tru64,
            97 => EiOsabi::Arm,
            255 => EiOsabi::Standalone,
            _ => EiOsabi::None,
        }
    }
}

impl<'a> HeaderField for EIdent<'a> {
    type Value = Option<()>;
    fn describe(&self, _endian: &Endian) -> String {
        format!(
            "Magic: {}\nClass: {}\nData: {}\nVersion: {:?}\nOS/ABI: {}",
            bytes_to_hex(&*self.raw),
            self.ei_class().as_str(),
            self.ei_data().as_str(),
            self.ei_version(),
            self.ei_osabi().as_str()
        )
    }
    fn value(&self, _endian: &Endian) -> Self::Value {
        None
    }
}

impl<'a> From<&EIdent<'a>> for Vec<u8> {
    fn from(h: &EIdent) -> Vec<u8> {
        h.raw.to_vec()
    }
}

