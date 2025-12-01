extern crate capstone;

use capstone::prelude::*;
use capstone::{Syntax, Endian};
use tabled::{Table, Tabled};
use tabled::settings::{Settings, Remove,object::Rows, Style};
use crate::utils::bytes_to_hex::bytes_to_hex;
use anyhow::Result;

#[derive(Tabled)]
struct Instruction {
    address: String,
    ins: String,
    bytes: String,
}

pub fn disass(addr: u64, buf: &[u8]) -> Result<()> {
    let mut cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

    cs.
        set_syntax(Syntax::Intel)?;
    cs
        .set_endian(Endian::Little)?;

    let insns = cs.disasm_all(buf, addr)?;

    let table_config = Settings::default()
            .with(Style::empty())
            .with(Remove::row(Rows::first()));

    let mut instructions: Vec<Instruction> = Vec::with_capacity(insns.len());

    for i in insns.iter() {
        instructions.push(
            Instruction { 
                address: format!("0x{:X}", i.address()),
                bytes: bytes_to_hex(i.bytes()), 
                ins: format!("{} {}", i.mnemonic().unwrap(), i.op_str().unwrap())
            }
        );
    }

    let table = Table::new(instructions).with(table_config).to_string();
    println!("{table}");
    Ok(())
}
