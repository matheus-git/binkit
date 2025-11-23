use crate::dto::info_dto::InfoDTO;
use crate::elf64::Elf64Binary;
use crate::traits::binary::Binary;
use crate::elf64::printers::Elf64Printer;
use crate::traits::binary_printer::BinaryPrinter;
use std::error::Error;

pub struct InfoBinary<'a> {
    pub binary: &'a Elf64Binary,
    pub dto: InfoDTO<'a>
}

impl InfoBinary<'_> {
    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let printer: Elf64Printer = Elf64Printer;

        if self.dto.header {
            printer.print_header(self.binary.get_header());
        } else if self.dto.programs {
            printer.print_program_headers(self.binary.get_program_headers());
        } else if self.dto.sections {
            printer.print_section_headers(self.binary.get_section_headers());
        } else {
            return Err("Unknown argument!".into());
        }

        Ok(())
    }
}
