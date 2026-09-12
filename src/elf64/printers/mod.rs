use crate::presentation::{blank, heading, line};
use crate::traits::header_field::HeaderField;
use crate::utils::bytes_to_hex::bytes_to_hex;
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
    struct HeaderField {
        #[tabled(rename = "Field")]
        field: &'static str,
        #[tabled(rename = "Value")]
        value: String,
    }

    let byte_order = match endian {
        Endian::Little => "little-endian",
        Endian::Big => "big-endian",
    };
    let os_abi = match header.e_ident.raw[7] {
        0 => "System V",
        1 => "HP-UX",
        2 => "NetBSD",
        3 => "Linux",
        6 => "Solaris",
        8 => "IRIX",
        9 => "FreeBSD",
        10 => "Tru64",
        97 => "ARM",
        255 => "Standalone",
        _ => "Unknown",
    };

    heading(
        "ELF64",
        &format!(
            "{file} · {} · {byte_order}",
            header.e_machine.describe(endian)
        ),
    )?;
    let fields = [
        HeaderField {
            field: "Magic",
            value: bytes_to_hex(&header.e_ident.raw[0..4]),
        },
        HeaderField {
            field: "Class",
            value: "ELF64".to_string(),
        },
        HeaderField {
            field: "Byte order",
            value: byte_order.to_string(),
        },
        HeaderField {
            field: "Identification version",
            value: header.e_ident.raw[6].to_string(),
        },
        HeaderField {
            field: "OS/ABI",
            value: os_abi.to_string(),
        },
        HeaderField {
            field: "ABI version",
            value: header.e_ident.raw[8].to_string(),
        },
        HeaderField {
            field: "Type",
            value: header.e_type.describe(endian),
        },
        HeaderField {
            field: "Machine",
            value: header.e_machine.describe(endian),
        },
        HeaderField {
            field: "Version",
            value: header.e_version.describe(endian),
        },
        HeaderField {
            field: "Entry point",
            value: header.e_entry.describe(endian),
        },
        HeaderField {
            field: "Program header offset",
            value: format!("0x{:X}", header.e_phoff.value(endian)),
        },
        HeaderField {
            field: "Section header offset",
            value: format!("0x{:X}", header.e_shoff.value(endian)),
        },
        HeaderField {
            field: "Flags",
            value: format!("0x{:X}", header.e_flags.value(endian)),
        },
        HeaderField {
            field: "Header size",
            value: format!("{} B", header.e_ehsize.value(endian)),
        },
        HeaderField {
            field: "Program header size",
            value: format!("{} B", header.e_phentsize.value(endian)),
        },
        HeaderField {
            field: "Program header count",
            value: header.e_phnum.value(endian).to_string(),
        },
        HeaderField {
            field: "Section header size",
            value: format!("{} B", header.e_shentsize.value(endian)),
        },
        HeaderField {
            field: "Section header count",
            value: header.e_shnum.value(endian).to_string(),
        },
        HeaderField {
            field: "Section names index",
            value: format!("index {}", header.e_shstrndx.value(endian)),
        },
    ];
    let table = Table::new(fields)
        .with(Settings::default().with(Style::modern()))
        .to_string();
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
