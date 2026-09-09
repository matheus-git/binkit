use crate::presentation::{blank, heading, line};
use crate::traits::header_field::HeaderField;
use crate::utils::endian::Endian;
use crate::utils::read_cstring::read_cstring;

use super::types::{
    elf64_header::Elf64Header, elf64_program_header::Elf64ProgramHeader,
    elf64_section_header::Elf64SectionHeader,
};
use anyhow::{Context, Result};
use tabled::settings::{Settings, Style};
use tabled::{Table, Tabled};

pub fn print_header(header: &Elf64Header, endian: &Endian, file: &str) -> Result<()> {
    #[derive(Tabled)]
    struct HeaderField<'a> {
        #[tabled(rename = "Field")]
        name: &'a str,
        #[tabled(rename = "Value")]
        describe: String,
    }

    let fields = vec![
        HeaderField {
            name: "Identification",
            describe: header.e_ident.describe(endian),
        },
        HeaderField {
            name: "Type",
            describe: header.e_type.describe(endian),
        },
        HeaderField {
            name: "Machine",
            describe: header.e_machine.describe(endian),
        },
        HeaderField {
            name: "Version",
            describe: header.e_version.describe(endian),
        },
        HeaderField {
            name: "Entry point",
            describe: header.e_entry.describe(endian),
        },
        HeaderField {
            name: "Program table offset",
            describe: header.e_phoff.describe(endian),
        },
        HeaderField {
            name: "Section table offset",
            describe: header.e_shoff.describe(endian),
        },
        HeaderField {
            name: "Flags",
            describe: header.e_flags.describe(endian),
        },
        HeaderField {
            name: "Header size",
            describe: header.e_ehsize.describe(endian),
        },
        HeaderField {
            name: "Program entry size",
            describe: header.e_phentsize.describe(endian),
        },
        HeaderField {
            name: "Program headers",
            describe: header.e_phnum.describe(endian),
        },
        HeaderField {
            name: "Section entry size",
            describe: header.e_shentsize.describe(endian),
        },
        HeaderField {
            name: "Section headers",
            describe: header.e_shnum.describe(endian),
        },
        HeaderField {
            name: "Section names index",
            describe: header.e_shstrndx.describe(endian),
        },
    ];

    let table_config = Settings::default().with(Style::modern());
    let table = Table::new(fields).with(table_config).to_string();
    heading("ELF64 header", file)?;
    line(format_args!("{table}"))?;
    Ok(())
}

pub fn print_program_headers(phs: &[Elf64ProgramHeader], endian: &Endian) -> Result<()> {
    #[derive(Tabled)]
    #[allow(clippy::struct_field_names)]
    struct ProgramHeaderFields {
        #[tabled(rename = "Type")]
        p_type: String,
        #[tabled(rename = "Offset")]
        p_offset: String,
        #[tabled(rename = "Virtual address")]
        p_vaddr: String,
        #[tabled(rename = "Physical address")]
        p_paddr: String,
        #[tabled(rename = "File size")]
        p_filesz: String,
        #[tabled(rename = "Memory size")]
        p_memsz: String,
        #[tabled(rename = "Flags")]
        p_flags: String,
        #[tabled(rename = "Alignment")]
        p_align: String,
    }

    let table_config = Settings::default().with(Style::modern());

    let mut fields: Vec<ProgramHeaderFields> = Vec::with_capacity(phs.len());

    for ph in phs {
        fields.push(ProgramHeaderFields {
            p_type: ph.p_type.describe(endian),
            p_offset: ph.p_offset.describe(endian),
            p_vaddr: ph.p_vaddr.describe(endian),
            p_paddr: ph.p_paddr.describe(endian),
            p_filesz: ph.p_filesz.describe(endian),
            p_memsz: ph.p_memsz.describe(endian),
            p_flags: ph.p_flags.describe(endian),
            p_align: ph.p_align.describe(endian),
        });
    }

    let table = Table::new(fields).with(table_config).to_string();

    blank()?;
    heading("Program headers", &format!("{} total", phs.len()))?;
    line(format_args!("{table}"))?;
    Ok(())
}

pub fn print_section_headers(
    shs: &[Elf64SectionHeader],
    endian: &Endian,
    strtab: &[u8],
) -> Result<()> {
    #[derive(Tabled)]
    #[allow(clippy::struct_field_names)]
    struct SectionHeaderFields {
        #[tabled(rename = "Name")]
        sh_name: String,
        #[tabled(rename = "Type")]
        sh_type: String,
        #[tabled(rename = "Flags")]
        sh_flags: String,
        #[tabled(rename = "Address")]
        sh_addr: String,
        #[tabled(rename = "Offset")]
        sh_offset: String,
        #[tabled(rename = "Size")]
        sh_size: String,
        #[tabled(rename = "Link")]
        sh_link: String,
        #[tabled(rename = "Info")]
        sh_info: String,
        #[tabled(rename = "Alignment")]
        sh_addralign: String,
        #[tabled(rename = "Entry size")]
        sh_entsize: String,
    }

    let table_config = Settings::default().with(Style::modern());

    let mut fields: Vec<SectionHeaderFields> = Vec::with_capacity(shs.len());

    for sh in shs {
        let name_offset = usize::try_from(sh.sh_name.value(endian))
            .context("Section name offset does not fit in usize")?;
        let raw_name = strtab
            .get(name_offset..)
            .context("Section name offset is outside the string table")?;
        let sh_name = read_cstring(raw_name)?;
        fields.push(SectionHeaderFields {
            sh_name: sh_name.to_string(),
            sh_type: sh.sh_type.describe(endian),
            sh_flags: sh.sh_flags.describe(endian),
            sh_addr: sh.sh_addr.describe(endian),
            sh_offset: sh.sh_offset.describe(endian),
            sh_size: sh.sh_size.describe(endian),
            sh_link: sh.sh_link.describe(endian),
            sh_info: sh.sh_info.describe(endian),
            sh_addralign: sh.sh_addralign.describe(endian),
            sh_entsize: sh.sh_entsize.describe(endian),
        });
    }

    let table = Table::new(fields).with(table_config).to_string();

    blank()?;
    heading("Section headers", &format!("{} total", shs.len()))?;
    line(format_args!("{table}"))?;
    Ok(())
}
