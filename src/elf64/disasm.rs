use crate::dto::disasm_dto::DisasmDTO;
use crate::elf64::Elf64Binary;
use crate::traits::binary::Binary;
use crate::disasm::disass;
use std::error::Error;

pub struct DisasmBinary<'a> {
    pub binary: &'a Elf64Binary,
    pub dto: DisasmDTO<'a>
}

impl DisasmBinary<'_> {
    fn get_bytes_section(&self, section_name: &str) -> Result<(u64, &[u8]), String> {
        let section = self.binary.get_section_headers()
            .iter()
            .find(|s| s.sh_name.name == section_name)
            .ok_or_else(|| format!("Section '{}' not found", section_name))?;

        let endian = self.binary.endian();
        let offset = endian.read_u64(section.sh_offset.raw) as usize;
        let size = endian.read_u64(section.sh_size.raw) as usize;

        Ok((endian.read_u64(section.sh_addr.raw), &self.binary.raw[offset..offset + size]))
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let section = self.dto.section.unwrap_or(".text");

        let (addr, bytes) = self.get_bytes_section(section)?;
        disass(addr, &bytes);

        Ok(())
    }
}
