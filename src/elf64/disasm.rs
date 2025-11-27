use crate::elf64::types::elf64_section_header::Elf64SectionHeader;
use crate::{dto::disasm_dto::DisasmDTO, utils::endian::Endian};
use crate::traits::header_field::HeaderField;
use crate::elf64::Elf64Binary;
use crate::traits::binary::Binary;
use crate::disasm::disass;
use anyhow::{Context, Result, anyhow};

pub struct DisasmBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: DisasmDTO<'a>
}

impl DisasmBinary<'_> {
    fn get_section(&self, section_name: &str, endian: &Endian) -> Result<&Elf64SectionHeader> {
        for section in self.binary.get_section_headers().iter() {
            let current_section_name = self.binary
                .resolve_section_name(section, endian)
                .with_context(|| format!("Failed resolving name for section"))?;

            if current_section_name == section_name {
                return Ok(section);
            }
        }

        Err(anyhow!("Section '{}' not found", section_name))
    }
    
    fn get_bytes_section(&self, section_name: &str) -> Result<(u64, &[u8])> {
        let endian = &self.binary.endian();
        let section = self.get_section(section_name, endian)?;

        let offset = usize::try_from(section.sh_offset.value(endian))
            .context("Offset does not fit in usize")?;

        let size = usize::try_from(section.sh_size.value(endian))
            .context("Size does not fit in usize")?;

        let end = offset + size;
        if end > self.binary.raw.len() {
            return Err(anyhow!(
                "Section '{}' exceeds binary bounds (0x{:x}..0x{:x})",
                section_name, offset, end,
            ));
        }

        Ok((section.sh_addr.value(endian), &self.binary.raw[offset..end]))
    }

    pub fn execute(&self) -> Result<()> {
        let section = self.dto.section.unwrap_or(".text");

        let (addr, bytes) = self.get_bytes_section(section)?;
        disass(addr, bytes);

        Ok(())
    }
}
