use crate::dto::update_dto::UpdateDTO;
use crate::elf64::Elf64Binary;
use crate::presentation::{blank, field, heading, success};
use crate::utils::parse_hex::parse_hex_to_u64;
use crate::utils::save_file::save_file;
use anyhow::{Result, anyhow};
use std::borrow::Cow;

pub struct UpdateBinary<'a> {
    pub binary: &'a mut Elf64Binary<'a>,
    pub dto: UpdateDTO<'a>,
}

impl UpdateBinary<'_> {
    pub fn set_entry(&mut self, hex_entry: &str) -> Result<()> {
        let endian = self.binary.endian();

        let entry = parse_hex_to_u64(hex_entry)?;
        self.binary.header.e_entry.raw = Cow::Owned(endian.to_bytes_u64(entry));
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let final_output = self.dto.output.unwrap_or(self.dto.file);

        if let Some(entry) = self.dto.entry {
            self.set_entry(entry)?;
            let bytes: Vec<u8> = (&*self.binary).try_into()?;
            let overwrite = self.dto.force || final_output == self.dto.file;
            save_file(final_output, &bytes, overwrite, Some(self.dto.file))?;
            heading("Update complete", self.dto.file)?;
            field("Entry point", entry)?;
            field("Output", final_output)?;
            blank()?;
            success("Output written atomically · permissions preserved")?;
        } else {
            return Err(anyhow!("Not found arguments"));
        }

        Ok(())
    }
}
