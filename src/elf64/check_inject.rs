use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::elf64::{Elf64Binary, calculate_rel32};
use anyhow::{Result, Context};

pub struct CheckInjectBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: CheckInjectDTO<'a>
}

impl CheckInjectBinary<'_> {
    pub fn execute(&self) -> Result<()> {
        let default_return_address = self.binary.entry();
        let return_address = if let Some(s) = self.dto.return_address {
            u64::from_str_radix(s.trim_start_matches("0x"), 16)
                .context("Invalid hexadecimal value for return_address")?
        } else {
            default_return_address
        };

        let addr = self.binary.get_address_to_inject()
            .context("Failed to determine injection address")?;
        println!("Injection slot available at: 0x{addr:X}");

        let rel32_addr = calculate_rel32(addr, return_address)?;
        match self.dto.return_address {
            Some(_) => println!("Rel32 relative to 0x{return_address:X}: 0x{rel32_addr:X}"),
            None => println!("Rel32 to original entry point (0x{return_address:X}): 0x{rel32_addr:X}")
        }

        Ok(())
    }
}
