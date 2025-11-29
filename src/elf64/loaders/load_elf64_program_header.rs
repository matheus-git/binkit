extern crate plain;
use plain::Plain;
use anyhow::{Result, anyhow};

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
#[allow(clippy::struct_field_names)]
pub struct LoadELF64ProgramHeader {
    pub p_type: [u8; 4],
    pub p_flags: [u8; 4],
    pub p_offset: [u8; 8],
    pub p_vaddr: [u8; 8],
    pub p_paddr: [u8; 8],
    pub p_filesz: [u8; 8],
    pub p_memsz: [u8; 8],
    pub p_align: [u8; 8],
}

unsafe impl Plain for LoadELF64ProgramHeader {}

impl LoadELF64ProgramHeader {
    pub fn from_bytes(buf: &[u8]) -> Result<&LoadELF64ProgramHeader> {
        plain::from_bytes(buf).map_err(|e| anyhow!("{e:?}"))
    }
}
