use crate::dto::inject_dto::InjectDTO;
use crate::elf64::{Elf64Binary};
use crate::utils::parse_hex::parse_hex_to_u64;
use crate::utils::save_file::save_file;
use anyhow::{Result, Context};
use std::fs;

pub struct InjectBinary<'a> {
    pub binary: &'a mut Elf64Binary<'a>,
    pub dto: InjectDTO<'a>
}

impl<'a> InjectBinary<'a> {
    fn update_section_name(&mut self, section_name_idx: usize){
        //let endian = self.binary.header.e_ident.endian();

        //let shstrtab_idx = endian.read_u16(self.binary.header.e_shstrndx.raw);
        //let shstrtab_section_header = &self.binary.section_headers[shstrtab_idx as usize];
        //let shstrtab_section_header_offset = endian.read_u64(shstrtab_section_header.sh_offset.raw);
        //
        //let new_name = ".injected\0".as_bytes(); 
        //let start = shstrtab_section_header_offset as usize + section_name_idx;
        //let end = start + new_name.len();
        //
        //self.binary.raw[start..end].copy_from_slice(new_name);
    }

    fn inject(&mut self, buf: Vec<u8>, _new_addr: u64, section: &str) -> Vec<u8> {
        //let target_section: &str = section;

        buf

        //let bytes: Vec<u8> = self.binary.into();
        //let file_off = bytes.len() as u64;

        //let endian = self.binary.header.e_ident.endian();

        //let note_section = self.binary.section_headers
        //    .iter_mut()
        //    .find(|s| s.sh_name.name == target_section);

        //let note_offset = if let Some(section) = note_section {
        //    let note_offset = section.sh_offset.raw;
        //    let section_name_idx = endian.read_u32(section.sh_name.raw) as usize;

        //    section.sh_type.raw = endian.to_bytes_u32(1);            
        //    section.sh_addr.raw = endian.to_bytes_u64(new_addr);
        //    section.sh_size.raw = endian.to_bytes_u64(buf.len() as  u64);
        //    section.sh_offset.raw = endian.to_bytes_u64(file_off);
        //    section.sh_addralign.raw = endian.to_bytes_u64(16);
        //    section.sh_flags.raw = endian.to_bytes_u64(6);

        //    self.update_section_name(section_name_idx);

        //    note_offset
        //} else {
        //    println!("{} not found", target_section);
        //    return Vec::new();
        //};        

        //if let Some(program) = self.binary.program_headers
        //    .iter_mut()
        //    .find(|p| p.p_offset.raw == note_offset)
        //{
        //    program.p_offset.raw = endian.to_bytes_u64(self.binary.raw.len() as u64);
        //    program.p_flags.raw = endian.to_bytes_u32(5);
        //    program.p_type.raw = endian.to_bytes_u32(1);
        //    program.p_vaddr.raw = endian.to_bytes_u64(new_addr);
        //    program.p_paddr.raw = endian.to_bytes_u64(new_addr);
        //    program.p_memsz.raw = endian.to_bytes_u64(buf.len() as u64);
        //    program.p_filesz.raw = endian.to_bytes_u64(buf.len() as u64);
        //    program.p_align.raw = endian.to_bytes_u64(ALIGN);
        //} else {
        //    println!("Program header not found!");
        //    return Vec::new();
        //}

        //let mut injected: Vec<u8> = self.binary.into();
        //injected.extend(buf);

        //injected
    }

    pub fn execute(&mut self) -> Result<()> {
        //let bytes = fs::read(self.dto.inject)?; 

        //let address = self
        //    .dto
        //    .address
        //    .map(parse_hex_to_u64)
        //    .unwrap_or_else(|| self.binary.get_address_to_inject());

        //let return_address = self.dto.return_address
        //    .map(parse_hex_to_u64)
        //    .unwrap_or_else(|| self.binary.entry());

        //let section = self.dto.section.unwrap_or(".note.ABI-tag");

        //let injected: Vec<u8> = self.inject(bytes, address, section);
        //println!("Payload injected at 0x{:X}", address);
        //let rel32_addr = self.binary.calculate_rel32(address, return_address);

        //match self.dto.address {
        //    Some(_) => println!("Rel32 to 0x{:X}: 0x{:X}", return_address, rel32_addr),
        //    None => println!("Rel32 to original entry point (0x{:X}): 0x{:X}", return_address, rel32_addr)
        //}

        //save_file(self.dto.output, &injected)?;
        //println!("Output written to: {}", self.dto.output);

        Ok(())
    }
}
