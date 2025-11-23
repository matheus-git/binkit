use crate::dto::update_dto::UpdateDTO;
use crate::elf64::Elf64Binary;
use crate::utils::save_file::save_file;
use std::num::ParseIntError;
use std::error::Error;

pub struct UpdateBinary<'a> {
    pub binary: &'a mut Elf64Binary,
    pub dto: UpdateDTO<'a>
}

impl<'a> UpdateBinary<'a> {
    pub fn set_entry(&mut self, hex_entry: &str) -> Result<(), ParseIntError> {
        let endian = self.binary.endian();

        let entry = u64::from_str_radix(hex_entry.trim_start_matches("0x"), 16)?;
        self.binary.header.e_entry.raw = endian.to_bytes_u64(entry);
        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), Box<dyn Error>> {
        let final_output = self.dto.output.unwrap_or(self.dto.file);

        if let Some(entry) = self.dto.entry {
            self.set_entry(entry)?; 
            let bytes: Vec<u8> = (&*self.binary).into();
            save_file(final_output, &bytes)?;
            println!("Output written to: {}", final_output);
        }

        Ok(())
    }
}
