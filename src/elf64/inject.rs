use crate::dto::inject_dto::InjectDTO;
use crate::elf64::{Elf64Binary, ALIGN, calculate_rel32};
use crate::traits::header_field::HeaderField;
use crate::utils::save_file::save_file;
use anyhow::{Result, anyhow};
use std::fs;
use std::borrow::Cow;

pub struct InjectBinary<'a> {
    pub binary: &'a mut Elf64Binary<'a>,
    pub dto: InjectDTO<'a>
}

impl InjectBinary<'_> {
    fn update_section_name(&mut self, section_name_idx: usize) -> Result<()>{
        let endian = &self.binary.header.e_ident.endian();

        let shstrtab_idx = self.binary.header.e_shstrndx.value(endian);
        let shstrtab_section_header = &self.binary.section_headers[shstrtab_idx as usize];
        let shstrtab_section_header_offset = shstrtab_section_header.sh_offset.value(endian);
        
        let new_name = ".injected\0".as_bytes(); 
        let shstrtab_section_header_offset = usize::try_from(shstrtab_section_header_offset)?;
        let start = shstrtab_section_header_offset.checked_add(section_name_idx).ok_or(anyhow!("failed".to_string()))?;
        let end = start + new_name.len();
        
        self.binary.raw.to_mut()[start..end].copy_from_slice(new_name);
        Ok(())
    }

    fn inject(&mut self, buf: &[u8], new_addr: u64, section: &str) -> Result<()> {
        let target_section: &str = section;
        let endian = &self.binary.header.e_ident.endian();

        let section_index = self.binary.section_headers.iter().position(|s| {
            self.binary.resolve_section_name(s, endian).ok() == Some(target_section)
        }).ok_or_else(|| anyhow::anyhow!("Section not found"))?;

        let note_section = &mut self.binary.section_headers[section_index];

        let note_offset = note_section.sh_offset.raw.clone();
        let section_name_idx = note_section.sh_name.value(endian) as usize;

        note_section.sh_type.raw = Cow::Owned(endian.to_bytes_u32(1));            
        note_section.sh_addr.raw = Cow::Owned(endian.to_bytes_u64(new_addr));
        note_section.sh_size.raw = Cow::Owned(endian.to_bytes_u64(buf.len() as  u64));
        note_section.sh_offset.raw = Cow::Owned(endian.to_bytes_u64(self.binary.raw.len().try_into()?));
        note_section.sh_addralign.raw = Cow::Owned(endian.to_bytes_u64(16));
        note_section.sh_flags.raw = Cow::Owned(endian.to_bytes_u64(6));

        self.update_section_name(section_name_idx)?;

        if let Some(program) = self.binary.program_headers
            .iter_mut()
            .find(|p| p.p_offset.raw == note_offset)
        {
            program.p_offset.raw = Cow::Owned(endian.to_bytes_u64(self.binary.raw.len() as u64));
            program.p_flags.raw = Cow::Owned(endian.to_bytes_u32(5));
            program.p_type.raw = Cow::Owned(endian.to_bytes_u32(1));
            program.p_vaddr.raw = Cow::Owned(endian.to_bytes_u64(new_addr));
            program.p_paddr.raw = Cow::Owned(endian.to_bytes_u64(new_addr));
            program.p_memsz.raw = Cow::Owned(endian.to_bytes_u64(buf.len() as u64));
            program.p_filesz.raw = Cow::Owned(endian.to_bytes_u64(buf.len() as u64));
            program.p_align.raw = Cow::Owned(endian.to_bytes_u64(ALIGN));
        } else {
            println!("Program header not found!");
        }

        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let bytes = fs::read(self.dto.inject)?; 

        let address = match self.dto.address {
            Some(a) => u64::from_str_radix(a, 16)?,  
            None => self.binary.get_address_to_inject()?,
        };

        let return_address = match self.dto.return_address {
            Some(a) => u64::from_str_radix(a, 16)?,  
            None => self.binary.entry(),
        };

        let section = self.dto.section.unwrap_or(".note.ABI-tag");

        self.inject(&bytes, address, section)?;
        let injected: Vec<u8> = (&*self.binary).try_into()?;
        println!("Payload injected at 0x{address:X}");
        let rel32_addr = calculate_rel32(address, return_address)?;

        match self.dto.address {
            Some(_) => println!("Rel32 to 0x{return_address:X}: 0x{rel32_addr:X}"),
            None => println!("Rel32 to original entry point (0x{return_address:X}): 0x{rel32_addr:X}")
        }

        save_file(self.dto.output, &injected)?;
        println!("Output written to: {}", self.dto.output);

        Ok(())
    }
}
