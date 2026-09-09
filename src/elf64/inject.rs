use crate::dto::inject_dto::InjectDTO;
use crate::elf64::{ALIGN, Elf64Binary, calculate_rel32};
use crate::presentation::{accent_field, blank, field, heading, success};
use crate::traits::header_field::HeaderField;
use crate::utils::parse_hex::parse_hex_to_u64;
use crate::utils::save_file::save_file;
use anyhow::{Context, Result, anyhow};
use std::borrow::Cow;
use std::fs;

pub struct InjectBinary<'a> {
    pub binary: &'a mut Elf64Binary<'a>,
    pub dto: InjectDTO<'a>,
}

impl InjectBinary<'_> {
    fn update_section_name(&mut self, section_name_idx: usize) -> Result<()> {
        let endian = &self.binary.header.e_ident.endian();

        let shstrtab_idx = usize::from(self.binary.header.e_shstrndx.value(endian));
        let shstrtab_section_header = self
            .binary
            .section_headers
            .get(shstrtab_idx)
            .context("String table section index is out of bounds")?;
        let shstrtab_section_header_offset = shstrtab_section_header.sh_offset.value(endian);

        let new_name = ".injected\0".as_bytes();
        let shstrtab_section_header_offset = usize::try_from(shstrtab_section_header_offset)?;
        let shstrtab_size = usize::try_from(shstrtab_section_header.sh_size.value(endian))?;
        let table_end = shstrtab_section_header_offset
            .checked_add(shstrtab_size)
            .context("String table range overflow")?;
        let start = shstrtab_section_header_offset
            .checked_add(section_name_idx)
            .context("Section name offset overflow")?;
        let end = start
            .checked_add(new_name.len())
            .context("Section name range overflow")?;

        let table = self
            .binary
            .raw
            .get(shstrtab_section_header_offset..table_end)
            .context("String table is outside the file bounds")?;
        let current_name = table
            .get(section_name_idx..)
            .context("Section name offset is outside the string table")?;
        let current_name_len = current_name
            .iter()
            .position(|byte| *byte == 0)
            .map(|length| length + 1)
            .context("Section name is not NUL-terminated")?;
        if current_name_len < new_name.len() {
            return Err(anyhow!(
                "Section name slot is too short to rename safely to .injected"
            ));
        }

        let write_range = section_name_idx..section_name_idx + new_name.len();
        let overlaps_another_name = self.binary.section_headers.iter().any(|section| {
            let other_start = section.sh_name.value(endian) as usize;
            other_start != section_name_idx
                && table
                    .get(other_start..)
                    .and_then(|name| name.iter().position(|byte| *byte == 0))
                    .is_some_and(|length| {
                        let other_range = other_start..other_start + length + 1;
                        write_range.start < other_range.end && other_range.start < write_range.end
                    })
        });
        if overlaps_another_name {
            return Err(anyhow!(
                "Renaming this section would corrupt another section name"
            ));
        }

        let destination = self
            .binary
            .raw
            .to_mut()
            .get_mut(start..end)
            .context("Not enough space in the string table for the injected section name")?;
        destination.copy_from_slice(new_name);
        Ok(())
    }

    fn inject(&mut self, buf: &[u8], new_addr: u64, section: &str) -> Result<()> {
        let target_section: &str = section;
        let endian = &self.binary.header.e_ident.endian();

        let section_index = self
            .binary
            .section_headers
            .iter()
            .position(|s| self.binary.resolve_section_name(s, endian).ok() == Some(target_section))
            .ok_or_else(|| anyhow::anyhow!("Section not found"))?;

        let note_section = &mut self.binary.section_headers[section_index];

        let note_offset = note_section.sh_offset.raw.clone();
        let section_name_idx = note_section.sh_name.value(endian) as usize;
        let raw_offset = u64::try_from(self.binary.raw.len())?;
        let payload_size = u64::try_from(buf.len())?;
        let program_index = self
            .binary
            .program_headers
            .iter()
            .position(|program| program.p_offset.raw == note_offset)
            .ok_or_else(|| {
                anyhow!(
                    "Program header not found! {}",
                    note_section.sh_offset.describe(endian)
                )
            })?;

        self.update_section_name(section_name_idx)?;

        let note_section = &mut self.binary.section_headers[section_index];
        note_section.sh_type.raw = Cow::Owned(endian.to_bytes_u32(1));
        note_section.sh_addr.raw = Cow::Owned(endian.to_bytes_u64(new_addr));
        note_section.sh_size.raw = Cow::Owned(endian.to_bytes_u64(payload_size));
        note_section.sh_offset.raw = Cow::Owned(endian.to_bytes_u64(raw_offset));
        note_section.sh_addralign.raw = Cow::Owned(endian.to_bytes_u64(16));
        note_section.sh_flags.raw = Cow::Owned(endian.to_bytes_u64(6));

        let program = &mut self.binary.program_headers[program_index];
        program.p_offset.raw = Cow::Owned(endian.to_bytes_u64(raw_offset));
        program.p_flags.raw = Cow::Owned(endian.to_bytes_u32(5));
        program.p_type.raw = Cow::Owned(endian.to_bytes_u32(1));
        program.p_vaddr.raw = Cow::Owned(endian.to_bytes_u64(new_addr));
        program.p_paddr.raw = Cow::Owned(endian.to_bytes_u64(new_addr));
        program.p_memsz.raw = Cow::Owned(endian.to_bytes_u64(payload_size));
        program.p_filesz.raw = Cow::Owned(endian.to_bytes_u64(payload_size));
        program.p_align.raw = Cow::Owned(endian.to_bytes_u64(ALIGN));

        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        self.binary.ensure_x86_64_little_endian("Injection")?;
        let bytes = fs::read(self.dto.inject)?;

        let address = match self.dto.address {
            Some(a) => parse_hex_to_u64(a)?,
            None => self.binary.get_address_to_inject()?,
        };

        let return_address = match self.dto.return_address {
            Some(a) => parse_hex_to_u64(a)?,
            None => self.binary.entry(),
        };

        let section = self.dto.section.unwrap_or(".note.gnu.property");

        self.inject(&bytes, address, section)?;
        let mut injected: Vec<u8> = (&*self.binary).try_into()?;
        injected.extend_from_slice(&bytes);
        let rel32_addr = calculate_rel32(address, return_address)?;

        save_file(
            self.dto.output,
            &injected,
            self.dto.force,
            Some(self.dto.file),
        )?;
        heading("Injection complete", self.dto.file)?;
        field(
            "Payload",
            format_args!("{} ({} bytes)", self.dto.inject, bytes.len()),
        )?;
        field("Section", ".injected")?;
        accent_field("Virtual address", format_args!("0x{address:016X}"))?;
        field("Return address", format_args!("0x{return_address:016X}"))?;
        field("Return rel32", format_args!("{rel32_addr:+#010X}"))?;
        field("Output", self.dto.output)?;
        blank()?;
        success("Output written atomically · permissions preserved")?;

        Ok(())
    }
}
