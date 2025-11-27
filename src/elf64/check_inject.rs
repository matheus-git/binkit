use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::elf64::Elf64Binary;
use crate::utils::bytes_to_hex::bytes_to_hex;
use crate::utils::parse_hex::parse_hex_to_u64;
use anyhow::{Result, Context};

pub struct CheckInjectBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: CheckInjectDTO<'a>
}

impl CheckInjectBinary<'_> {
    pub fn execute(&self) -> Result<()> {
        let endian = self.binary.endian();
        let default_return_address = bytes_to_hex(&endian.to_bytes_u64(self.binary.entry()));
        let return_address = self.dto.return_address.unwrap_or(
            default_return_address.as_str()
        );

        let addr = self.binary.get_address_to_inject()
            .context("esfsdfsda")?;
        println!("Injection slot available at: 0x{:X}", addr);

        let return_address_u64 = parse_hex_to_u64(return_address);
        let rel32_addr = self.binary.calculate_rel32(addr, return_address_u64);
        match self.dto.return_address {
            Some(_) => println!("Rel32 relative to 0x{:X}: 0x{:X}", return_address_u64, rel32_addr),
            None => println!("Rel32 to original entry point (0x{:X}): 0x{:X}", return_address_u64, rel32_addr)
        }

        Ok(())
    }
}
