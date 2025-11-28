use crate::dto::update_dto::UpdateDTO;
use crate::elf64::Elf64Binary;
use crate::utils::save_file::save_file;
use std::borrow::Cow;
use anyhow::{Result, anyhow};

pub struct UpdateBinary<'a> {
    pub binary: &'a mut Elf64Binary<'a>,
    pub dto: UpdateDTO<'a>
}

impl UpdateBinary<'_> {
    pub fn set_entry(&mut self, hex_entry: &str) -> Result<()> {
        let endian = self.binary.endian();

        let entry = u64::from_str_radix(hex_entry.strip_prefix("0x").unwrap_or(hex_entry), 16)?;
        self.binary.header.e_entry.raw = Cow::Owned(endian.to_bytes_u64(entry));
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let final_output = self.dto.output.unwrap_or(self.dto.file);

        if let Some(entry) = self.dto.entry {
            self.set_entry(entry)?; 
            let bytes: Vec<u8> = (&*self.binary).try_into()?;
            save_file(final_output, &bytes)?;
            println!("Output written to: {final_output}");
        } else {
            return Err(anyhow!("Not found arguments"));
        }

        Ok(())
    }
}
