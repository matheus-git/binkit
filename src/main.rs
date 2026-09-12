use binkit::elf64::Elf64Binary;
use binkit::{
    CheckInjectDTO, DisasmDTO, InfoDTO, InjectDTO, MappedFile, UpdateDTO, disass_with_count,
};

use std::fs;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

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

        #[arg(
            short = 'i',
            long,
            help = "Path to the file containing bytes to inject"
        )]
        inject: String,

        #[arg(
            short = 'a',
            long,
            help = "Virtual address to assign to the injected payload (hexadecimal)"
        )]
        address: Option<String>,

        #[arg(
            short = 's',
            long,
            help = "Section header to repurpose (default: .note.gnu.property)"
        )]
        section: Option<String>,

        #[arg(
            short = 'r',
            long,
            help = "Return address after injected code executes (hexadecimal). Defaults to ELF entry point"
        )]
        return_address: Option<String>,

        #[arg(short = 'o', long, help = "Path to save the modified ELF output")]
        output: String,

        #[arg(short = 'f', long, help = "Overwrite the output file if it exists")]
        force: bool,
    },

    #[command(about = "Check available injection point in an ELF file")]
    CheckInject {
        #[arg(help = "Path to the ELF file to analyze")]
        file: String,

        #[arg(
            short = 'r',
            long,
            help = "Return address for calculating relative offsets (hexadecimal). Defaults to ELF entry point"
        )]
        return_address: Option<String>,
    },

    #[command(about = "Disassemble a section or address range of an ELF file")]
    Disasm {
        #[arg(help = "Path to the ELF file to disassemble")]
        file: String,

        #[arg(short = 'b', long, help = "Read the file as raw binary")]
        bin: bool,

        #[arg(
            short = 's',
            long,
            help = "Section name to disassemble. (default: .text) "
        )]
        section: Option<String>,

        #[arg(
            long,
            conflicts_with = "address",
            help = "Start at this byte offset within the selected input"
        )]
        offset: Option<usize>,

        #[arg(
            long,
            conflicts_with = "offset",
            help = "Start at this virtual address (hexadecimal, ELF input only)"
        )]
        address: Option<String>,

        #[arg(long, help = "Decode at most this many instructions")]
        count: Option<usize>,

        #[arg(long, help = "Read at most this many bytes from the selected start")]
        bytes: Option<usize>,
    },

    #[command(about = "Display ELF file information")]
    Info {
        #[arg(help = "Path to the ELF file to analyze")]
        file: String,

        #[arg(short = 'H', long, help = "Display the ELF header information")]
        header: bool,

        #[arg(
            short = 'p',
            long,
            help = "Display the program headers of the ELF file"
        )]
        programs: bool,

        #[arg(
            short = 's',
            long,
            help = "Display the section headers of the ELF file"
        )]
        sections: bool,
    },

    #[command(about = "Update ELF metadata")]
    Update {
        #[arg(help = "Path to the ELF file to modify")]
        file: String,

        #[arg(
            short = 'e',
            long,
            help = "Set a new entry point for the ELF file (hexadecimal)"
        )]
        entry: Option<String>,

        #[arg(short = 'o', long, help = "Path to save the updated ELF file")]
        output: Option<String>,

        #[arg(short = 'f', long, help = "Overwrite the output file if it exists")]
        force: bool,
    },
}

fn load_file(file: &str) -> Result<Vec<u8>> {
    Ok(fs::read(file)?)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let raw: Vec<u8>;
    let mut binary: Elf64Binary;

    match &cli.command {
        Commands::Inject {
            file,
            address,
            return_address,
            inject,
            output,
            section,
            force,
        } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = InjectDTO {
                file,
                inject,
                address: address.as_deref(),
                section: section.as_deref(),
                return_address: return_address.as_deref(),
                output,
                force: *force,
            };

            let mut inject = binary.inject(dto);

            inject.execute().context("Inject failed")?;
        }
        Commands::CheckInject {
            file,
            return_address,
        } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = CheckInjectDTO {
                file,
                return_address: return_address.as_deref(),
            };

            let check_inject = binary.check_inject(dto);

            check_inject
                .execute()
                .context("Check inject command failed")?;
        }
        Commands::Disasm {
            file,
            section,
            bin,
            offset,
            address,
            count,
            bytes,
        } => {
            if *bin {
                if address.is_some() {
                    return Err(anyhow::anyhow!("--address requires an ELF input"));
                }
                let mapped = MappedFile::open(file)?;
                let input = mapped.as_ref();
                let start = offset.unwrap_or(0);
                if start > input.len() {
                    return Err(anyhow::anyhow!(
                        "Start offset 0x{start:X} is outside the input ({} bytes)",
                        input.len()
                    ));
                }
                let available = input.len() - start;
                let length = bytes.unwrap_or(available).min(available);
                let end = start
                    .checked_add(length)
                    .context("Disassembly byte range overflows usize")?;
                disass_with_count(start as u64, &input[start..end], *count)?;
                return Ok(());
            }

            let mapped = MappedFile::open(file)?;
            binary = Elf64Binary::new(mapped.as_ref())?;

            let dto = DisasmDTO {
                file,
                section: section.as_deref(),
                offset: *offset,
                address: address.as_deref(),
                count: *count,
                bytes: *bytes,
            };

            let disasm = binary.disasm(dto);

            disasm.execute().context("Disasm command failed")?;
        }
        Commands::Info {
            file,
            header,
            programs,
            sections,
        } => {
            let mapped = MappedFile::open(file)?;
            binary = Elf64Binary::new(mapped.as_ref())?;

            let dto = InfoDTO {
                file,
                header: *header,
                programs: *programs,
                sections: *sections,
            };

            let info = binary.info(dto);

            info.execute().context("Info command failed")?;
        }
        Commands::Update {
            file,
            entry,
            output,
            force,
        } => {
            raw = load_file(file)?;
            binary = Elf64Binary::new(&raw)?;

            let dto = UpdateDTO {
                file,
                entry: entry.as_deref(),
                output: output.as_deref(),
                force: *force,
            };

            let mut update = binary.update(dto);

            update.execute().context("Update command failed")?;
        }
    }

    Ok(())
}
