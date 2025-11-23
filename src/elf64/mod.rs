mod loaders;
pub mod types;
pub mod printers;
pub mod disasm;
pub mod update;

use disasm::DisasmBinary;
use update::UpdateBinary;
use std::cmp::max;
use loaders::load_elf64_header::LoadELF64Header;
use loaders::load_elf64_program_header::LoadELF64ProgramHeader;
use loaders::load_elf64_section_header::LoadELF64SectionHeader;

use types::elf64_header::Elf64Header;
use types::elf64_program_header::Elf64ProgramHeader;
use types::elf64_section_header::Elf64SectionHeader;
use crate::dto::update_dto::UpdateDTO;
use crate::traits::binary::Binary;
use crate::utils::endian::Endian;
use crate::utils::string_until_null::string_until_null;
use crate::dto::disasm_dto::DisasmDTO;

fn parse_program_headers(buf: &[u8], elf_header: &Elf64Header, endian: &Endian) -> Vec<Elf64ProgramHeader> {
    let mut headers = Vec::with_capacity(elf_header.e_phnum.value as usize);

    for i in 0..elf_header.e_phnum.value as usize {
        let start = elf_header.e_phoff.value as usize + i * elf_header.e_phentsize.value as usize;
        let end = start + elf_header.e_phentsize.value as usize;

        let raw_header = LoadELF64ProgramHeader::from_bytes(&buf[start..end]);
        headers.push(Elf64ProgramHeader::new(raw_header, endian));
    }

    headers
}

fn parse_section_headers(buf: &[u8], elf_header: &Elf64Header, endian: &Endian) -> Vec<Elf64SectionHeader> {
    let mut headers = Vec::with_capacity(elf_header.e_shnum.value as usize);

    for i in 0..elf_header.e_shnum.value as usize {
        let start = elf_header.e_shoff.value as usize + i * elf_header.e_shentsize.value as usize;
        let end = start + elf_header.e_shentsize.value as usize;

        let raw_header = LoadELF64SectionHeader::from_bytes(&buf[start..end]);
        headers.push(Elf64SectionHeader::new(raw_header, endian));
    }

    headers
}

fn resolve_section_name(section_headers: &mut Vec<Elf64SectionHeader>, buf: &[u8], elf_header: &Elf64Header, endian: &Endian){
    let strtab_section = &section_headers[elf_header.e_shstrndx.value as usize ];
    let strtab_section_offset = endian.read_u64(strtab_section.sh_offset.raw);

    for section in section_headers {
        let name_index = section.sh_name.value;
        let name = string_until_null(&buf[(strtab_section_offset as usize + name_index as usize)..]);
        section.sh_name.update_name(name.to_string());
    }
}

const ALIGN: u64 = 0x1000;

#[derive(Debug)]
pub struct Elf64Binary {
    header: Elf64Header,
    program_headers: Vec<Elf64ProgramHeader>,
    section_headers: Vec<Elf64SectionHeader>,
    raw: Vec<u8>
}

impl Elf64Binary {
    pub fn new(buf: &[u8]) -> Self{
        let load_elf_header =  LoadELF64Header::from_bytes(buf);
        let elf_header = Elf64Header::new(load_elf_header);
        let endian: Endian = elf_header.e_ident.endian();
        
        let program_headers = parse_program_headers(buf, &elf_header, &endian);
        let mut section_headers = parse_section_headers(buf, &elf_header, &endian);

        resolve_section_name(&mut section_headers, buf, &elf_header, &endian);

        Self { 
            header: elf_header, 
            program_headers,
            section_headers,
            raw: buf.to_vec()
        }
    }

    pub fn endian(&self) -> Endian {
        self.header.e_ident.endian()
    }

    pub fn disasm<'a>(&'a self, dto: DisasmDTO<'a>) -> DisasmBinary<'a> {
        DisasmBinary {
            binary: self,
            dto
        }
    }

    pub fn update<'a>(&'a mut self, dto: UpdateDTO<'a>) -> UpdateBinary<'a> {
        UpdateBinary {
            binary: self,
            dto
        }
    }

    pub fn entry(&self) -> u64 {
        let endian = self.endian();
        return endian.read_u64(self.header.e_entry.raw);
    }

    pub fn get_address_to_inject(&self) -> u64 {
        let program_headers = &self.program_headers;
        let endian = self.endian();
        let mut higher_addr: u64 = 0;
        for program in program_headers {
            let initial_address = endian.read_u64(program.p_vaddr.raw);
            let memsz = max(
                endian.read_u64(program.p_memsz.raw),
                endian.read_u64(program.p_filesz.raw)
            );
            let final_address = initial_address + memsz;
            if final_address > higher_addr {
                higher_addr = final_address;
            }
        };
        self.calculate_new_addr(higher_addr + ALIGN)
    }

    pub fn calculate_new_addr(&self, addr: u64) -> u64 {
        let bytes: Vec<u8> = self.into();
        let offset = bytes.len() as u64;
        let delta = (offset % ALIGN + ALIGN - (addr as u64 % ALIGN)) % ALIGN;
        addr + delta
    }

    pub fn calculate_rel32(&self, addr_base: u64, addr_target: u64) -> i64 {
        return addr_target as i64 - addr_base as i64;
    }

    pub fn update_section_name(&mut self, section_name_idx: usize){
        let endian = self.header.e_ident.endian();

        let shstrtab_idx = endian.read_u16(self.header.e_shstrndx.raw);
        let shstrtab_section_header = &self.section_headers[shstrtab_idx as usize];
        let shstrtab_section_header_offset = endian.read_u64(shstrtab_section_header.sh_offset.raw);
        
        let new_name = ".injected\0".as_bytes(); 
        let start = shstrtab_section_header_offset as usize + section_name_idx;
        let end = start + new_name.len();
        
        self.raw[start..end].copy_from_slice(new_name);
    }

    pub fn inject(&mut self, buf: Vec<u8>, new_addr: u64, section: &str) -> Vec<u8> {
        let target_section: &str = section;

        let bytes: Vec<u8> = self.into();
        let file_off = bytes.len() as u64;

        let endian = self.header.e_ident.endian();

        let note_section = self.section_headers
            .iter_mut()
            .find(|s| s.sh_name.name == target_section)
            .map(|s| s);

        let note_offset = if let Some(section) = note_section {
            let note_offset = section.sh_offset.raw;
            let section_name_idx = endian.read_u32(section.sh_name.raw) as usize;

            section.sh_type.raw = endian.to_bytes_u32(1);            
            section.sh_addr.raw = endian.to_bytes_u64(new_addr);
            section.sh_size.raw = endian.to_bytes_u64(buf.len() as  u64);
            section.sh_offset.raw = endian.to_bytes_u64(file_off);
            section.sh_addralign.raw = endian.to_bytes_u64(16);
            section.sh_flags.raw = endian.to_bytes_u64(6);

            self.update_section_name(section_name_idx);

            note_offset
        } else {
            println!("{} not found", target_section);
            return Vec::new();
        };        

        if let Some(program) = self.program_headers
            .iter_mut()
            .find(|p| p.p_offset.raw == note_offset)
        {
            program.p_offset.raw = endian.to_bytes_u64(self.raw.len() as u64);
            program.p_flags.raw = endian.to_bytes_u32(5);
            program.p_type.raw = endian.to_bytes_u32(1);
            program.p_vaddr.raw = endian.to_bytes_u64(new_addr);
            program.p_paddr.raw = endian.to_bytes_u64(new_addr);
            program.p_memsz.raw = endian.to_bytes_u64(buf.len() as u64);
            program.p_filesz.raw = endian.to_bytes_u64(buf.len() as u64);
            program.p_align.raw = endian.to_bytes_u64(ALIGN);
        } else {
            println!("Program header not found!");
            return Vec::new();
        }

        let mut injected: Vec<u8> = self.into();
        injected.extend(buf);

        injected
    }
}

impl Binary for Elf64Binary {
    type Header = Elf64Header;
    type ProgramHeader = Elf64ProgramHeader;
    type SectionHeader = Elf64SectionHeader;

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

impl From<&Elf64Binary> for Vec<u8> {
    fn from(h: &Elf64Binary) -> Vec<u8> {
        let mut bytes = h.raw.clone();

        let header_bytes: Vec<u8> = (&h.header).into();
        bytes[0..header_bytes.len()].copy_from_slice(&header_bytes);

        for (i, ph) in h.program_headers.iter().enumerate() {
            let ph_bytes: Vec<u8> = ph.into();
            let offset = h.header.e_phoff.value as usize + i * h.header.e_phentsize.value as usize;
            bytes[offset..offset + ph_bytes.len()].copy_from_slice(&ph_bytes);
        }

        for (i, sh) in h.section_headers.iter().enumerate() {
            let sh_bytes: Vec<u8> = sh.into();
            let offset = h.header.e_shoff.value as usize + i * h.header.e_shentsize.value as usize;
            bytes[offset..offset + sh_bytes.len()].copy_from_slice(&sh_bytes);
        }

        bytes
    }
}

impl From<&mut Elf64Binary> for Vec<u8> {
    fn from(h: &mut Elf64Binary) -> Vec<u8> {
        let mut bytes = h.raw.clone();

        let header_bytes: Vec<u8> = (&h.header).into();
        bytes[0..header_bytes.len()].copy_from_slice(&header_bytes);

        for (i, ph) in h.program_headers.iter().enumerate() {
            let ph_bytes: Vec<u8> = ph.into();
            let offset = h.header.e_phoff.value as usize + i * h.header.e_phentsize.value as usize;
            bytes[offset..offset + ph_bytes.len()].copy_from_slice(&ph_bytes);
        }

        for (i, sh) in h.section_headers.iter().enumerate() {
            let sh_bytes: Vec<u8> = sh.into();
            let offset = h.header.e_shoff.value as usize + i * h.header.e_shentsize.value as usize;
            bytes[offset..offset + sh_bytes.len()].copy_from_slice(&sh_bytes);
        }

        bytes
    }
}
