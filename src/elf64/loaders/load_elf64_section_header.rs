extern crate plain;
use plain::Plain;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
#[allow(clippy::struct_field_names)]
pub struct LoadELF64SectionHeader {
    pub sh_name: [u8; 4],
    pub sh_type: [u8; 4],
    pub sh_flags: [u8; 8],
    pub sh_addr: [u8; 8],
    pub sh_offset: [u8; 8],
    pub sh_size: [u8; 8],
    pub sh_link: [u8; 4],
    pub sh_info: [u8; 4],
    pub sh_addralign: [u8; 8],
    pub sh_entsize: [u8; 8],
}

unsafe impl Plain for LoadELF64SectionHeader {}

impl LoadELF64SectionHeader {
    pub fn from_bytes(buf: &[u8]) -> &LoadELF64SectionHeader {
        plain::from_bytes(buf).expect("Buffer too short or not aligned")
    }
}

