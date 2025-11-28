mod elf64;
mod traits;
mod utils;
mod disasm;
mod dto;

use elf64::Elf64Binary;
use dto::disasm_dto::DisasmDTO;

use std::fs;

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};

use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::dto::info_dto::InfoDTO;
use crate::dto::inject_dto::InjectDTO;
use crate::dto::update_dto::UpdateDTO;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Inject bytes into an ELF file")]
    Inject {
        #[arg(help = "Path to the ELF file to inject into")]
        file: String,

        #[arg(short = 'i', long, help = "Path to the file containing bytes to inject")]
        inject: String,

        #[arg(short = 'a', long, help = "Address to inject at (hexadecimal). Overrides section if provided")]
        address: Option<String>,

        #[arg(short = 's', long, help = "Section name to inject into. Used if address is not provided")]
        section: Option<String>,

        #[arg(short = 'r', long, help = "Return address after injected code executes (hexadecimal). Defaults to ELF entry point")]
        return_address: Option<String>,

        #[arg(short = 'o', long, help = "Path to save the modified ELF output")]
        output: String,
    },

    #[command(about = "Check available injection point in an ELF file")]
    CheckInject {
        #[arg(help = "Path to the ELF file to analyze")]
        file: String,

        #[arg(short = 'r', long, help = "Return address for calculating relative offsets (hexadecimal). Defaults to ELF entry point")]
        return_address: Option<String>,
    },

    #[command(about = "Disassemble a section or address range of an ELF file")]
    Disasm {
        #[arg(help = "Path to the ELF file to disassemble")]
        file: String,

        #[arg(short = 's', long, help = "Section name to disassemble. (default: .text) ")]
        section: Option<String>,
    },

    #[command(about = "Display ELF file information")]
    Info {
        #[arg(help = "Path to the ELF file to analyze")]
        file: String,

        #[arg(short = 'H', long, help = "Display the ELF header information")]
        header: bool,

        #[arg(short = 'p', long, help = "Display the program headers of the ELF file")]
        programs: bool,

        #[arg(short = 's', long, help = "Display the section headers of the ELF file")]
        sections: bool,
    },

    #[command(about = "Update ELF metadata")]
    Update {
        #[arg(help = "Path to the ELF file to modify")]
        file: String,       

        #[arg(short = 'e', long, help = "Set a new entry point for the ELF file (hexadecimal)")]
        entry: Option<String>,

        #[arg(short = 'o', long, help = "Path to save the updated ELF file")]
        output: Option<String>,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    fn load_file(file: &str) -> Result<Vec<u8>> {
        Ok(fs::read(file)?)
    }

    let raw: Vec<u8>;
    let mut binary: Elf64Binary;

    match &cli.command {
        Commands::Inject { file, address, return_address, inject, output, section } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = InjectDTO {
                file,
                inject,
                address: address.as_deref(),
                section: section.as_deref(),
                return_address: return_address.as_deref(),
                output
            };

            let mut inject = binary.inject(dto);

            inject.execute()
                .context("Inject failed")?;
        },
        Commands::CheckInject { file, return_address } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = CheckInjectDTO {
                file,
                return_address: return_address.as_deref()
            };

            let check_inject = binary.check_inject(dto);

            check_inject.execute()
                .context("Check inject command failed")?;
        },
        Commands::Disasm { file, section } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = DisasmDTO {
                file,
                section: section.as_deref(),
            };

            let disasm = binary.disasm(dto);

            disasm.execute()
                .context("Disasm command failed")?;
        },
        Commands::Info { file, header, programs, sections } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = InfoDTO {
                file, 
                header: *header,
                programs: *programs, 
                sections: *sections
            };

            let info = binary.info(dto);

            info.execute()
                .context("Info command failed")?;
        },
        Commands::Update { file, entry, output } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = UpdateDTO {
                file,
                entry: entry.as_deref(),
                output: output.as_deref()
            };

            let mut update = binary.update(dto);

            update.execute()
                .context("Update command failed")?;
        }
    }

    Ok(())
}
