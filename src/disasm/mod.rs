extern crate capstone;

use capstone::prelude::*;
use capstone::{Syntax, Endian};
use tabled::{Table, Tabled};
use tabled::settings::{Settings, Remove,object::Rows, Style};
use crate::utils::bytes_to_hex::bytes_to_hex;

#[derive(Tabled)]
struct Instruction {
    address: String,
    ins: String,
    bytes: String,
}

pub fn disass(addr: u64, buf: &[u8]) {
    let mut cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

    let _ = cs
        .set_endian(Endian::Little);

    let _ = cs.
        set_syntax(Syntax::Intel);

    let _ = cs. 
        set_detail(true);

    let _ = cs. 
        set_skipdata(false);

    let insns = cs.disasm_all(buf, addr)
        .expect("Failed to disassemble");

    println!("Found {} instructions\n", insns.len());

    let table_config = Settings::default()
            .with(Style::empty())
            .with(Remove::row(Rows::first()));

    let mut instructions: Vec<Instruction> = Vec::with_capacity(insns.len());

    for i in insns.as_ref() {
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
}
