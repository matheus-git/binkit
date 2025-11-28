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
use anyhow::{Context, Result};
use std::convert::TryFrom;

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

fn parse_program_headers<'a>(buf: &'a [u8], elf_header: &Elf64Header, endian: &Endian) -> Result<Vec<Elf64ProgramHeader<'a>>> {
    let phnum = elf_header.e_phnum.value(endian) as usize;
    let phoff = usize::try_from(elf_header.e_phoff.value(endian))
        .context("Failed to read the program header offset")?;
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

    Ok(headers)
}

fn parse_section_headers<'a>(buf: &'a [u8], elf_header: &Elf64Header, endian: &Endian) -> Result<Vec<Elf64SectionHeader<'a>>> {
    let shnum = elf_header.e_shnum.value(endian) as usize;
    let shoff = usize::try_from(elf_header.e_shoff.value(endian))
        .context("Failed to read the section header offset")?;
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

    Ok(headers)
}

pub const ALIGN: u64 = 0x1000;

pub fn calculate_rel32(addr_base: u64, addr_target: u64) -> Result<i32> {
    let diff = i128::from(addr_target) - i128::from(addr_base);
    i32::try_from(diff).context("rel32 does not fit in i32")
}

#[derive(Debug)]
pub struct Elf64Binary<'a> {
    header: Elf64Header<'a>,
    program_headers: Vec<Elf64ProgramHeader<'a>>,
    section_headers: Vec<Elf64SectionHeader<'a>>,
    raw: Cow<'a, [u8]>
}

impl<'a> Elf64Binary<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let load_elf_header =  LoadELF64Header::from_bytes(buf);
        let elf_header = Elf64Header::new(load_elf_header);
        let endian: Endian = elf_header.e_ident.endian();
        
        let program_headers = parse_program_headers(buf, &elf_header, &endian)?;
        let section_headers = parse_section_headers(buf, &elf_header, &endian)?;

        Ok(
            Self { 
                header: elf_header, 
                program_headers,
                section_headers,
                raw: Cow::Borrowed(buf)
            }
        )
    }

    pub fn resolve_section_name(&self, section: &Elf64SectionHeader, endian: &Endian) -> Result<&str>{
        let strtab_section_index = self.header.e_shstrndx.value(endian) as usize;
        let strtab_section = self.section_headers
            .get(strtab_section_index)
            .context("String table section index is out of bounds")?;

        let strtab_section_offset = usize::try_from(strtab_section.sh_offset.value(endian))
            .context("strtab offset does not fit in usize")?;
        let sh_name_index = usize::try_from(section.sh_name.value(endian))
            .context("Section name index does not fit in usize")?;

        let start = strtab_section_offset+sh_name_index;

        let raw_name = &self.raw
            .get(start..)
            .context("Section name offset is outside the file bounds")?;

        let name = read_cstring(raw_name)
            .context("Invalid section name")?;
        Ok(name)
    }

    pub fn endian(&self) -> Endian {
        self.header.e_ident.endian()
    }

    pub fn disasm(&'a self, dto: DisasmDTO<'a>) -> DisasmBinary<'a> {
        DisasmBinary {
            binary: self,
            dto
        }
    }

    pub fn update(&'a mut self, dto: UpdateDTO<'a>) -> UpdateBinary<'a> {
        UpdateBinary {
            binary: self,
            dto
        }
    }

    pub fn info(&'a self, dto: InfoDTO<'a>) -> InfoBinary<'a> {
        InfoBinary { 
            binary: self, 
            dto 
        }
    }

    pub fn check_inject(&'a self, dto: CheckInjectDTO<'a>) -> CheckInjectBinary<'a> {
        CheckInjectBinary { 
            binary: self, 
            dto 
        }
    }

    pub fn inject(&'a mut self, dto: InjectDTO<'a>) -> InjectBinary<'a> {
        InjectBinary { 
            binary: self, 
            dto 
        }
    }

    pub fn entry(&self) -> u64 {
        let endian = self.endian();
        endian.read_u64(*self.header.e_entry.raw)
    }

    pub fn get_address_to_inject(&self) -> Result<u64> {
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

    pub fn calculate_new_addr(&self, addr: u64) -> Result<u64> {
        let bytes: Vec<u8> = self.try_into()
            .context("dsfdsfds")?;
        let offset = bytes.len() as u64;
        let delta = (offset % ALIGN + ALIGN - (addr % ALIGN)) % ALIGN;
        Ok(addr + delta)
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

impl<'a> TryFrom<&'a Elf64Binary<'a>> for Vec<u8> {
    type Error = anyhow::Error;
    
    #[allow(clippy::too_many_lines)]
    fn try_from(h: &'a Elf64Binary<'a>) -> Result<Self, Self::Error> {
    let mut bytes: Vec<u8> = Vec::with_capacity(h.raw.len());
        let endian = &h.endian();

        let mut current_offset: usize = 0;

        let header_bytes: Vec<u8> = (&h.header).into();
        bytes.resize(header_bytes.len(), 0);
        bytes[current_offset..header_bytes.len()].copy_from_slice(&header_bytes);

        current_offset = header_bytes.len();

        let phoff = usize::try_from(h.header.e_phoff.value(endian))
            .context("sdfsdfsd")?;
        let shoff = usize::try_from(h.header.e_shoff.value(endian))
            .context("sfdsfsdd")?;

        let ph_first = phoff < shoff;

        if ph_first {
            if current_offset < phoff {
                let slice = &h.raw
                    .get(current_offset..phoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sdfsdfsd")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = phoff;
            }

            for ph in h.get_program_headers() {
                let ph_bytes: Vec<u8> = ph.into();
                let new_len = current_offset.checked_add(ph_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&ph_bytes);
                current_offset = new_len;
            }

            if current_offset < shoff {
                let slice = h.raw
                    .get(current_offset..shoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sfsdfds")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = shoff;
            }

            for sh in h.get_section_headers() {
                let sh_bytes: Vec<u8> = sh.into();
                let new_len = current_offset.checked_add(sh_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&sh_bytes);
                current_offset = new_len;
            }
        } else {
            if current_offset < shoff {
                let slice = &h.raw
                    .get(current_offset..shoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sdfsdfsd")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = shoff;
            }

            for sh in h.get_section_headers() {
                let sh_bytes: Vec<u8> = sh.into();
                let new_len = current_offset.checked_add(sh_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&sh_bytes);
                current_offset = new_len;
            }

            if current_offset < phoff {
                let slice = h.raw
                    .get(current_offset..phoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sfsdfds")?;
                bytes.resize(new_len, 0);
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = phoff;
            }

            for ph in h.get_program_headers() {
                let ph_bytes: Vec<u8> = ph.into();
                let new_len = current_offset.checked_add(ph_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&ph_bytes);
                current_offset = new_len;
            }

        }

        let slice = h.raw
            .get(current_offset..)
            .context("sdfsdfds")?;
        let new_len = current_offset.checked_add(slice.len())
            .context("overflow copying section header")?;
        if bytes.len() < new_len {
            bytes.resize(new_len, 0);
        }
        bytes[current_offset..].copy_from_slice(slice);

        Ok(bytes)

    }
}

impl<'a> TryFrom<&'a mut Elf64Binary<'a>> for Vec<u8> {
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(h: &'a mut Elf64Binary<'a>) -> Result<Self, Self::Error> {
    let mut bytes: Vec<u8> = Vec::with_capacity(h.raw.len());
        let endian = &h.endian();

        let mut current_offset: usize = 0;

        let header_bytes: Vec<u8> = (&h.header).into();
        bytes.resize(header_bytes.len(), 0);
        bytes[current_offset..header_bytes.len()].copy_from_slice(&header_bytes);

        current_offset = header_bytes.len();

        let phoff = usize::try_from(h.header.e_phoff.value(endian))
            .context("sdfsdfsd")?;
        let shoff = usize::try_from(h.header.e_shoff.value(endian))
            .context("sfdsfsdd")?;

        let ph_first = phoff < shoff;

        if ph_first {
            if current_offset < phoff {
                let slice = &h.raw
                    .get(current_offset..phoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sdfsdfsd")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = phoff;
            }

            for ph in h.get_program_headers() {
                let ph_bytes: Vec<u8> = ph.into();
                let new_len = current_offset.checked_add(ph_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&ph_bytes);
                current_offset = new_len;
            }

            if current_offset < shoff {
                let slice = h.raw
                    .get(current_offset..shoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sfsdfds")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = shoff;
            }

            for sh in h.get_section_headers() {
                let sh_bytes: Vec<u8> = sh.into();
                let new_len = current_offset.checked_add(sh_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&sh_bytes);
                current_offset = new_len;
            }
        } else {
            if current_offset < shoff {
                let slice = &h.raw
                    .get(current_offset..shoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sdfsdfsd")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = shoff;
            }

            for sh in h.get_section_headers() {
                let sh_bytes: Vec<u8> = sh.into();
                let new_len = current_offset.checked_add(sh_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&sh_bytes);
                current_offset = new_len;
            }

            if current_offset < phoff {
                let slice = h.raw
                    .get(current_offset..phoff)
                    .context("dsfsdfsd")?;
                let new_len = current_offset.checked_add(slice.len())
                    .context("sfsdfds")?;
                bytes.resize(new_len, 0);
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = phoff;
            }

            for ph in h.get_program_headers() {
                let ph_bytes: Vec<u8> = ph.into();
                let new_len = current_offset.checked_add(ph_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&ph_bytes);
                current_offset = new_len;
            }
        }

        let slice = h.raw
            .get(current_offset..)
            .context("sdfsdfds")?;
        let new_len = current_offset.checked_add(slice.len())
            .context("overflow copying section header")?;
        if bytes.len() < new_len {
            bytes.resize(new_len, 0);
        }
        bytes[current_offset..].copy_from_slice(slice);

        Ok(bytes)

    }
}
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
