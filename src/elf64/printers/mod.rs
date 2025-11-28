use crate::traits::header_field::HeaderField;
use crate::utils::endian::Endian;

use super::types::{elf64_header::Elf64Header, elf64_program_header::Elf64ProgramHeader, elf64_section_header::Elf64SectionHeader};
use tabled::{Table, Tabled};
use tabled::settings::{Settings, Style};

pub fn print_header(header: &Elf64Header, endian: &Endian) {
    #[derive(Tabled)]
    struct HeaderField<'a> {
        name: &'a str,
        describe: String,
    }

    let fields = vec![
        HeaderField { name: "e_ident",  describe: header.e_ident.describe(endian) },
        HeaderField { name: "e_type",  describe: header.e_type.describe(endian) },
        HeaderField { name: "e_machine",  describe: header.e_machine.describe(endian) },
        HeaderField { name: "e_version",  describe: header.e_version.describe(endian) },
        HeaderField { name: "e_entry",  describe: header.e_entry.describe(endian) },
        HeaderField { name: "e_phoff",  describe: header.e_phoff.describe(endian) },
        HeaderField { name: "e_shoff",  describe: header.e_shoff.describe(endian) },
        HeaderField { name: "e_flags",  describe: header.e_flags.describe(endian) },
        HeaderField { name: "e_ehsize",  describe: header.e_ehsize.describe(endian) },
        HeaderField { name: "e_phentsize",  describe: header.e_phentsize.describe(endian) },
        HeaderField { name: "e_phnum",  describe: header.e_phnum.describe(endian) },
        HeaderField { name: "e_shentsize",  describe: header.e_shentsize.describe(endian) },
        HeaderField { name: "e_shnum",  describe: header.e_shnum.describe(endian) },
        HeaderField { name: "e_shstrndx",  describe: header.e_shstrndx.describe(endian) },
    ];

    let table_config = Settings::default()
        .with(Style::modern());
    let table = Table::new(fields).with(table_config).to_string();
    println!("Elf header:");
    println!("{table}");
}

pub fn print_program_headers(phs: &[Elf64ProgramHeader], endian: &Endian) {
    #[derive(Tabled)]
    #[allow(clippy::struct_field_names)]
    struct ProgramHeaderFields {
        p_type: String,
        p_offset: String,
        p_vaddr: String,
        p_paddr: String,
        p_filesz: String,
        p_memsz: String,
        p_flags: String,
        p_align: String,
    }

    let table_config = Settings::default()
        .with(Style::modern());

    let mut fields: Vec<ProgramHeaderFields> = Vec::with_capacity(phs.len());

    for ph in phs {
        fields.push(
            ProgramHeaderFields { 
                p_type: ph.p_type.describe(endian),
                p_offset: ph.p_offset.describe(endian),
                p_vaddr: ph.p_vaddr.describe(endian),
                p_paddr: ph.p_paddr.describe(endian),
                p_filesz: ph.p_filesz.describe(endian),
                p_memsz: ph.p_memsz.describe(endian),
                p_flags: ph.p_flags.describe(endian),
                p_align: ph.p_align.describe(endian)
            }
        );
    }

    let table = Table::new(fields).with(table_config).to_string();

    println!("\nProgram headers:");
    println!("{table}");
}

pub fn print_section_headers(shs: &[Elf64SectionHeader], endian: &Endian) {
    #[derive(Tabled)]
    #[allow(clippy::struct_field_names)]
    struct SectionHeaderFields {
        sh_name: String,
        sh_type: String,
        sh_flags: String,
        sh_addr: String,
        sh_offset: String,
        sh_size: String,
        sh_link: String,
        sh_info: String,
        sh_addralign: String,
        sh_entsize: String,
    }

    let table_config = Settings::default()
        .with(Style::modern());

    let mut fields: Vec<SectionHeaderFields> = Vec::with_capacity(shs.len());

    for sh in shs {
        fields.push(
            SectionHeaderFields { 
                sh_name: sh.sh_name.describe(endian),
                sh_type: sh.sh_type.describe(endian),
                sh_flags: sh.sh_flags.describe(endian),
                sh_addr: sh.sh_addr.describe(endian),
                sh_offset: sh.sh_offset.describe(endian),
                sh_size: sh.sh_size.describe(endian),
                sh_link: sh.sh_link.describe(endian),
                sh_info: sh.sh_info.describe(endian),
                sh_addralign: sh.sh_addralign.describe(endian),
                sh_entsize: sh.sh_entsize.describe(endian)
            }
        );
    }

    let table = Table::new(fields).with(table_config).to_string();

    println!("\nSection headers:");
    println!("{table}");
}
