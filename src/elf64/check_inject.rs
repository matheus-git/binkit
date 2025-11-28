use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::elf64::Elf64Binary;
use anyhow::{Result, Context};

pub struct CheckInjectBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: CheckInjectDTO<'a>
}

impl CheckInjectBinary<'_> {
    pub fn execute(&self) -> Result<()> {
        let default_return_address = self.binary.entry();
        let return_address = if let Some(s) = self.dto.return_address.as_deref() {
            u64::from_str_radix(s.trim_start_matches("0x"), 16)
                .context("invalid hex in return_address")?
        } else {
            default_return_address
        };

        let addr = self.binary.get_address_to_inject()
            .context("esfsdfsda")?;
        println!("Injection slot available at: 0x{:X}", addr);

        let rel32_addr = self.binary.calculate_rel32(addr, return_address)?;
        match self.dto.return_address {
            Some(_) => println!("Rel32 relative to 0x{:X}: 0x{:X}", return_address, rel32_addr),
            None => println!("Rel32 to original entry point (0x{:X}): 0x{:X}", return_address, rel32_addr)
        }

        Ok(())
    }
}
