use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::elf64::{Elf64Binary, calculate_rel32};
use crate::presentation::{accent_field, field, heading};
use crate::utils::parse_hex::parse_hex_to_u64;
use anyhow::{Context, Result};

pub struct CheckInjectBinary<'a> {
    pub binary: &'a Elf64Binary<'a>,
    pub dto: CheckInjectDTO<'a>,
}

impl CheckInjectBinary<'_> {
    pub fn execute(&self) -> Result<()> {
        let default_return_address = self.binary.entry();
        let return_address = if let Some(s) = self.dto.return_address {
            parse_hex_to_u64(s).context("Invalid hexadecimal value for return_address")?
        } else {
            default_return_address
        };

        let addr = self
            .binary
            .get_address_to_inject()
            .context("Failed to determine injection address")?;
        let start_delta = calculate_rel32(addr, return_address)?;
        heading("Injection plan", self.dto.file)?;
        accent_field("Virtual address", format_args!("0x{addr:016X}"))?;
        field("Return address", format_args!("0x{return_address:016X}"))?;
        field(
            "Start → return",
            format_args!("0x{:08X} ({start_delta:+})", start_delta as u32),
        )?;
        field(
            "JMP rel32",
            "start-to-return − JMP byte offset − 5-byte instruction size",
        )?;
        field("Example", "JMP at payload offset 40: subtract 45 (0x2D)")?;

        Ok(())
    }
}
