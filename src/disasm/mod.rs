extern crate capstone;

use crate::presentation::{blank, field, heading, line};
use crate::utils::bytes_to_hex::bytes_to_hex;
use anyhow::Result;
use capstone::prelude::*;
use capstone::{Endian, Syntax};
use tabled::settings::{Settings, Style};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Instruction {
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Instruction")]
    ins: String,
    #[tabled(rename = "Bytes")]
    bytes: String,
}

pub fn disass(addr: u64, buf: &[u8]) -> Result<()> {
    let mut cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .detail(true)
        .build()?;

    cs.set_syntax(Syntax::Intel)?;
    cs.set_endian(Endian::Little)?;

    let insns = cs.disasm_all(buf, addr)?;

    let table_config = Settings::default().with(Style::psql());

    let mut instructions: Vec<Instruction> = Vec::with_capacity(insns.len());

    for i in insns.iter() {
        instructions.push(Instruction {
            address: format!("0x{:X}", i.address()),
            bytes: bytes_to_hex(i.bytes()),
            ins: format!(
                "{} {}",
                i.mnemonic().unwrap_or("<unknown>"),
                i.op_str().unwrap_or("")
            ),
        });
    }

    heading("Disassembly", "x86-64 · Intel syntax")?;
    let instruction_count = instructions.len();
    let table = Table::new(instructions).with(table_config).to_string();
    line(format_args!("{table}"))?;
    blank()?;
    field("Instructions", instruction_count)?;
    field("Decoded bytes", buf.len())?;
    Ok(())
}
