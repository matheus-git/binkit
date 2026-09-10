extern crate plain;
use anyhow::{Result, anyhow};
use plain::Plain;

#[repr(C)]
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct LoadELF64Header {
    pub e_ident: [u8; 16],
    pub e_type: [u8; 2],
    pub e_machine: [u8; 2],
    pub e_version: [u8; 4],
    pub e_entry: [u8; 8],
    pub e_phoff: [u8; 8],
    pub e_shoff: [u8; 8],
    pub e_flags: [u8; 4],
    pub e_ehsize: [u8; 2],
    pub e_phentsize: [u8; 2],
    pub e_phnum: [u8; 2],
    pub e_shentsize: [u8; 2],
    pub e_shnum: [u8; 2],
    pub e_shstrndx: [u8; 2],
}

const _: () = {
    assert!(std::mem::size_of::<LoadELF64Header>() == 64);
    assert!(std::mem::align_of::<LoadELF64Header>() == 1);
};

// SAFETY: every field is a byte array, so every bit pattern is valid. The compile-time size and
// alignment assertions above guarantee the packed ELF64 byte layout expected by `from_bytes`.
unsafe impl Plain for LoadELF64Header {}

impl LoadELF64Header {
    pub fn from_bytes(buf: &[u8]) -> Result<&LoadELF64Header> {
        plain::from_bytes(buf).map_err(|e| anyhow!("{e:?}"))
    }
}
