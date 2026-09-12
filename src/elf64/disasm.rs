use crate::disasm::disass_with_count;
use crate::elf64::Elf64Binary;
use crate::elf64::types::elf64_section_header::Elf64SectionHeader;
use crate::traits::binary::Binary;
use crate::traits::header_field::HeaderField;
use crate::{
    dto::disasm_dto::DisasmDTO,
    utils::{endian::Endian, parse_hex::parse_hex_to_u64},
};
use anyhow::{Context, Result, anyhow};

pub struct DisasmBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: DisasmDTO<'a>,
}

impl DisasmBinary<'_> {
    fn get_section(&self, section_name: &str, endian: &Endian) -> Result<&Elf64SectionHeader<'_>> {
        for section in self.binary.get_section_headers() {
            let current_section_name = self
                .binary
                .resolve_section_name(section, endian)
                .with_context(|| "Failed resolving name for section".to_string())?;

            if current_section_name == section_name {
                return Ok(section);
            }
        }

        Err(anyhow!("Section '{section_name}' not found"))
    }

    fn get_bytes_section(&self, section_name: &str) -> Result<(u64, &[u8])> {
        let endian = &self.binary.endian();
        let section = self.get_section(section_name, endian)?;

        let offset = usize::try_from(section.sh_offset.value(endian))
            .context("Offset does not fit in usize")?;

        let size =
            usize::try_from(section.sh_size.value(endian)).context("Size does not fit in usize")?;

        let end = offset
            .checked_add(size)
            .context("Section range overflows the platform address size")?;
        if end > self.binary.raw.len() {
            return Err(anyhow!(
                "Section '{section_name}' exceeds binary bounds (0x{offset:x}..0x{end:x})"
            ));
        }

        Ok((section.sh_addr.value(endian), &self.binary.raw[offset..end]))
    }

    pub fn execute(&self) -> Result<()> {
        self.binary.ensure_x86_64_little_endian("Disassembly")?;
        let section = self.dto.section.unwrap_or(".text");

        let (section_addr, section_bytes) = self.get_bytes_section(section)?;
        let offset = if let Some(address) = self.dto.address {
            let address = parse_hex_to_u64(address).context("Invalid start address")?;
            let relative = address.checked_sub(section_addr).ok_or_else(|| {
                anyhow!("Address 0x{address:X} is before section '{section}' at 0x{section_addr:X}")
            })?;
            usize::try_from(relative).context("Address offset does not fit in usize")?
        } else {
            self.dto.offset.unwrap_or(0)
        };
        if offset > section_bytes.len() {
            return Err(anyhow!(
                "Start offset 0x{offset:X} is outside section '{section}' ({} bytes)",
                section_bytes.len()
            ));
        }
        let available = section_bytes.len() - offset;
        let length = self.dto.bytes.unwrap_or(available).min(available);
        let end = offset
            .checked_add(length)
            .context("Disassembly byte range overflows usize")?;
        let address = section_addr
            .checked_add(offset as u64)
            .context("Disassembly address overflows u64")?;

        disass_with_count(address, &section_bytes[offset..end], self.dto.count)?;

        Ok(())
    }
}
