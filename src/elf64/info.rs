use crate::elf64::Elf64Binary;
use crate::elf64::printers::{print_program_headers, print_section_headers};
use crate::traits::binary::Binary;
use crate::{dto::info_dto::InfoDTO, elf64::printers::print_header};
use anyhow::{Result, anyhow};

pub struct InfoBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: InfoDTO<'a>,
}

impl InfoBinary<'_> {
    pub fn execute(&self) -> Result<()> {
        let endian = &self.binary.endian();

        if !self.dto.header && !self.dto.programs && !self.dto.sections {
            return Err(anyhow!("At least one info option must be selected"));
        }

        if self.dto.header {
            print_header(self.binary.get_header(), endian, self.dto.file)?;
        }
        if self.dto.programs {
            print_program_headers(self.binary.get_program_headers(), endian)?;
        }
        if self.dto.sections {
            let strtab = self.binary.strtab()?;
            print_section_headers(self.binary.get_section_headers(), endian, strtab)?;
        }

        Ok(())
    }
}
