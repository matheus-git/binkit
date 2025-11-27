mod loaders;
pub mod types;
pub mod printers;
pub mod disasm;
pub mod update;
pub mod info;
pub mod check_inject;
pub mod inject;

use std::borrow::Cow;
use disasm::DisasmBinary;
use update::UpdateBinary;
use info::InfoBinary;
use inject::InjectBinary;
use std::cmp::max;
use loaders::load_elf64_header::LoadELF64Header;
use loaders::load_elf64_program_header::LoadELF64ProgramHeader;
use loaders::load_elf64_section_header::LoadELF64SectionHeader;
use anyhow::{Result, Context};

use types::elf64_header::Elf64Header;
use types::elf64_program_header::Elf64ProgramHeader;
use types::elf64_section_header::Elf64SectionHeader;
use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::dto::info_dto::InfoDTO;
use crate::dto::inject_dto::InjectDTO;
use crate::dto::update_dto::UpdateDTO;
use crate::elf64::check_inject::CheckInjectBinary;
use crate::traits::binary::Binary;
use crate::traits::header_field::HeaderField;
use crate::utils::endian::Endian;
use crate::utils::read_cstring::read_cstring;
use crate::dto::disasm_dto::DisasmDTO;

fn parse_program_headers<'a>(buf: &'a [u8], elf_header: &Elf64Header, endian: &Endian) -> Vec<Elf64ProgramHeader<'a>> {
    let phnum = elf_header.e_phnum.value(endian) as usize;
    let phoff = elf_header.e_phoff.value(endian) as usize;
    let phentsize = elf_header.e_phentsize.value(endian) as usize;
    let mut headers = Vec::with_capacity(phnum);

    for i in 0..phnum {
        let start = phoff + i * phentsize;
        let end = start + phentsize;

        if end > buf.len() {
            break;
        }

        let raw_header = LoadELF64ProgramHeader::from_bytes(&buf[start..end]);
        headers.push(Elf64ProgramHeader::new(raw_header));
    }

    headers
}

fn parse_section_headers<'a>(buf: &'a [u8], elf_header: &Elf64Header, endian: &Endian) -> Vec<Elf64SectionHeader<'a>> {
    let shnum = elf_header.e_shnum.value(endian) as usize;
    let shoff = elf_header.e_shoff.value(endian) as usize;
    let shentsize = elf_header.e_shentsize.value(endian) as usize;
    let mut headers = Vec::with_capacity(shnum);

    for i in 0..shnum {
        let start = shoff + i * shentsize;
        let end = start + shentsize;

        if end > buf.len() {
            break;
        }

        let raw_header = LoadELF64SectionHeader::from_bytes(&buf[start..end]);
        headers.push(Elf64SectionHeader::new(raw_header));
    }

    headers
}

pub const ALIGN: u64 = 0x1000;

#[derive(Debug)]
pub struct Elf64Binary<'a> {
    header: Elf64Header<'a>,
    program_headers: Vec<Elf64ProgramHeader<'a>>,
    section_headers: Vec<Elf64SectionHeader<'a>>,
    raw: Cow<'a, [u8]>
}

impl<'a> Elf64Binary<'a> {
    pub fn new(buf: &'a [u8]) -> Self{
        let load_elf_header =  LoadELF64Header::from_bytes(buf);
        let elf_header = Elf64Header::new(load_elf_header);
        let endian: Endian = elf_header.e_ident.endian();
        
        let program_headers = parse_program_headers(buf, &elf_header, &endian);
        let section_headers = parse_section_headers(buf, &elf_header, &endian);

        Self { 
            header: elf_header, 
            program_headers,
            section_headers,
            raw: Cow::Borrowed(buf)
        }
    }

    pub fn resolve_section_name(&self, section: &Elf64SectionHeader, endian: &Endian) -> Result<&str>{
        let strtab_section_index = usize::try_from(self.header.e_shstrndx.value(endian))
            .context("strtab section does not fit in usize")?;
        let strtab_section = &self.section_headers[strtab_section_index];
        let strtab_section_offset = usize::try_from(strtab_section.sh_offset.value(endian))
            .context("strtab offset does not fit in usize")?;

        let sh_name_index = usize::try_from(section.sh_name.value(endian))
            .context("Section name index does not fit in usize")?;

        let name = read_cstring(&self.raw[strtab_section_offset+sh_name_index..])
            .context("Invalid section name")?;
        Ok(name)
    }

    pub fn endian(&self) -> Endian {
        self.header.e_ident.endian()
    }

    pub fn disasm(&self, dto: DisasmDTO<'a>) -> DisasmBinary {
        DisasmBinary {
            binary: self,
            dto
        }
    }

    pub fn update(&'a mut self, dto: UpdateDTO<'a>) -> UpdateBinary {
        UpdateBinary {
            binary: self,
            dto
        }
    }

    pub fn info(&self, dto: InfoDTO) -> InfoBinary {
        InfoBinary { 
            binary: self, 
            dto 
        }
    }

    pub fn check_inject(&self, dto: CheckInjectDTO) -> CheckInjectBinary {
        CheckInjectBinary { 
            binary: self, 
            dto 
        }
    }

    pub fn inject(&mut self, dto: InjectDTO) -> InjectBinary {
        InjectBinary { 
            binary: self, 
            dto 
        }
    }

    pub fn entry(&self) -> u64 {
        let endian = self.endian();
        endian.read_u64(*self.header.e_entry.raw)
    }

    pub fn get_address_to_inject(&self) -> u64 {
        let program_headers = &self.program_headers;
        let endian = self.endian();
        let mut higher_addr: u64 = 0;
        for program in program_headers {
            let initial_address = endian.read_u64(*program.p_vaddr.raw);
            let memsz = max(
                endian.read_u64(*program.p_memsz.raw),
                endian.read_u64(*program.p_filesz.raw)
            );
            let final_address = initial_address + memsz;
            if final_address > higher_addr {
                higher_addr = final_address;
            }
        };
        self.calculate_new_addr(higher_addr + ALIGN)
    }

    pub fn calculate_new_addr(&self, addr: u64) -> u64 {
        //let bytes: Vec<u8> = self.into();
        let bytes: Vec<u8> = Vec::new();
        let offset = bytes.len() as u64;
        let delta = (offset % ALIGN + ALIGN - (addr % ALIGN)) % ALIGN;
        addr + delta
    }

    #[inline]
    pub fn calculate_rel32(&self, addr_base: u64, addr_target: u64) -> i64 {
        addr_target as i64 - addr_base as i64
    }

}

impl<'a> Binary for Elf64Binary<'a> {
    type Header = Elf64Header<'a>;
    type ProgramHeader = Elf64ProgramHeader<'a>;
    type SectionHeader = Elf64SectionHeader<'a>;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_program_headers(&self) -> &[Self::ProgramHeader] {
        &self.program_headers
    }

    fn get_section_headers(&self) -> &[Self::SectionHeader] {
        &self.section_headers
    }
}

//impl<'a> From<&Elf64Binary<'a>> for Vec<u8> {
//    fn from(h: Elf64Binary<'a>) -> Vec<u8> {
//        let mut bytes = h.raw.clone();
//        let endian = &h.endian();
//
//        let header_bytes: Vec<u8> = (h.header).into();
//        bytes[0..header_bytes.len()].copy_from_slice(&header_bytes);
//
//        for (i, ph) in h.program_headers.iter().enumerate() {
//            let ph_bytes: Vec<u8> = ph.into();
//            let offset = h.header.e_phoff.value(endian) as usize + i * h.header.e_phentsize.value(endian) as usize;
//            bytes[offset..offset + ph_bytes.len()].copy_from_slice(&ph_bytes);
//        }
//
//        for (i, sh) in h.section_headers.iter().enumerate() {
//            let sh_bytes: Vec<u8> = sh.into();
//            let offset = h.header.e_shoff.value(endian) as usize + i * h.header.e_shentsize.value(endian) as usize;
//            bytes[offset..offset + sh_bytes.len()].copy_from_slice(&sh_bytes);
//        }
//
//        bytes.to_vec()
//    }
//}
//
//impl From<&mut Elf64Binary> for Vec<u8> {
//    fn from(h: &mut Elf64Binary) -> Vec<u8> {
//        let mut bytes = h.raw.clone();
//
//        let header_bytes: Vec<u8> = (&h.header).into();
//        bytes[0..header_bytes.len()].copy_from_slice(&header_bytes);
//
//        for (i, ph) in h.program_headers.iter().enumerate() {
//            let ph_bytes: Vec<u8> = ph.into();
//            let offset = h.header.e_phoff.value as usize + i * h.header.e_phentsize.value as usize;
//            bytes[offset..offset + ph_bytes.len()].copy_from_slice(&ph_bytes);
//        }
//
//        for (i, sh) in h.section_headers.iter().enumerate() {
//            let sh_bytes: Vec<u8> = sh.into();
//            let offset = h.header.e_shoff.value as usize + i * h.header.e_shentsize.value as usize;
//            bytes[offset..offset + sh_bytes.len()].copy_from_slice(&sh_bytes);
//        }
//
//        bytes
//    }
//}
