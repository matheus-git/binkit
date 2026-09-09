mod check_inject;
mod disasm;
mod info;
mod inject;
mod loaders;
mod printers;
mod types;
mod update;

pub use check_inject::CheckInjectBinary;
pub use disasm::DisasmBinary;
pub use info::InfoBinary;
pub use inject::InjectBinary;
pub use update::UpdateBinary;

use anyhow::{Context, Result, anyhow};
use loaders::load_elf64_header::LoadELF64Header;
use loaders::load_elf64_program_header::LoadELF64ProgramHeader;
use loaders::load_elf64_section_header::LoadELF64SectionHeader;
use std::borrow::Cow;
use std::cmp::max;
use std::convert::TryFrom;
use std::ops::Range;

use crate::dto::check_inject_dto::CheckInjectDTO;
use crate::dto::disasm_dto::DisasmDTO;
use crate::dto::info_dto::InfoDTO;
use crate::dto::inject_dto::InjectDTO;
use crate::dto::update_dto::UpdateDTO;
use crate::traits::binary::Binary;
use crate::traits::header_field::HeaderField;
use crate::utils::endian::Endian;
use crate::utils::read_cstring::read_cstring;
use types::elf64_header::Elf64Header;
use types::elf64_program_header::Elf64ProgramHeader;
use types::elf64_section_header::Elf64SectionHeader;

fn parse_program_headers<'a>(
    buf: &'a [u8],
    elf_header: &Elf64Header,
    endian: &Endian,
) -> Result<Vec<Elf64ProgramHeader<'a>>> {
    let phnum = elf_header.e_phnum.value(endian) as usize;
    let phoff = usize::try_from(elf_header.e_phoff.value(endian))
        .context("Failed to read the program header offset")?;
    let phentsize = elf_header.e_phentsize.value(endian) as usize;

    if phnum > 0 && phentsize != std::mem::size_of::<LoadELF64ProgramHeader>() {
        return Err(anyhow!(
            "Invalid ELF64 program header entry size: {phentsize}"
        ));
    }

    let mut headers = Vec::with_capacity(phnum);

    for i in 0..phnum {
        let entry_offset = i
            .checked_mul(phentsize)
            .context("Program header table offset overflow")?;
        let start = phoff
            .checked_add(entry_offset)
            .context("Program header table offset overflow")?;
        let end = start
            .checked_add(phentsize)
            .context("Program header range overflow")?;
        let raw = buf
            .get(start..end)
            .context("ELF file is truncated in the program header table")?;

        let raw_header = LoadELF64ProgramHeader::from_bytes(raw)?;
        headers.push(Elf64ProgramHeader::new(raw_header));
    }

    Ok(headers)
}

fn parse_section_headers<'a>(
    buf: &'a [u8],
    elf_header: &Elf64Header,
    endian: &Endian,
) -> Result<Vec<Elf64SectionHeader<'a>>> {
    let shnum = elf_header.e_shnum.value(endian) as usize;
    let shoff = usize::try_from(elf_header.e_shoff.value(endian))
        .context("Failed to read the section header offset")?;
    let shentsize = elf_header.e_shentsize.value(endian) as usize;
    if shnum > 0 && shentsize != std::mem::size_of::<LoadELF64SectionHeader>() {
        return Err(anyhow!(
            "Invalid ELF64 section header entry size: {shentsize}"
        ));
    }
    let mut headers = Vec::with_capacity(shnum);

    for i in 0..shnum {
        let entry_offset = i
            .checked_mul(shentsize)
            .context("Section header table offset overflow")?;
        let start = shoff
            .checked_add(entry_offset)
            .context("Section header table offset overflow")?;
        let end = start
            .checked_add(shentsize)
            .context("Section header range overflow")?;
        let raw = buf
            .get(start..end)
            .context("ELF file is truncated in the section header table")?;

        let raw_header = LoadELF64SectionHeader::from_bytes(raw)?;
        headers.push(Elf64SectionHeader::new(raw_header));
    }

    Ok(headers)
}

fn table_range(
    offset: u64,
    count: u16,
    entry_size: u16,
    file_len: usize,
    table_name: &str,
) -> Result<Option<Range<usize>>> {
    if count == 0 {
        return Ok(None);
    }

    let start = usize::try_from(offset)
        .with_context(|| format!("{table_name} offset does not fit in usize"))?;
    if start < std::mem::size_of::<LoadELF64Header>() {
        return Err(anyhow!("{table_name} overlaps the ELF header"));
    }
    let length = usize::from(count)
        .checked_mul(usize::from(entry_size))
        .with_context(|| format!("{table_name} size overflow"))?;
    let end = start
        .checked_add(length)
        .with_context(|| format!("{table_name} range overflow"))?;
    if end > file_len {
        return Err(anyhow!("{table_name} is outside the file bounds"));
    }
    Ok(Some(start..end))
}

pub const ALIGN: u64 = 0x1000;

pub fn calculate_rel32(addr_base: u64, addr_target: u64) -> Result<i32> {
    let diff = i128::from(addr_target) - i128::from(addr_base);
    i32::try_from(diff).context("rel32 does not fit in i32")
}

#[derive(Debug)]
pub struct Elf64Binary<'a> {
    header: Elf64Header<'a>,
    program_headers: Vec<Elf64ProgramHeader<'a>>,
    section_headers: Vec<Elf64SectionHeader<'a>>,
    raw: Cow<'a, [u8]>,
}

impl<'a> Elf64Binary<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let load_elf_header = LoadELF64Header::from_bytes(buf)?;
        if &load_elf_header.e_ident[0..4] != b"\x7fELF" {
            return Err(anyhow!("Invalid ELF magic"));
        }
        if load_elf_header.e_ident[4] != 2 {
            return Err(anyhow!("Only ELF64 binaries are supported"));
        }
        if !matches!(load_elf_header.e_ident[5], 1 | 2) {
            return Err(anyhow!("Invalid ELF byte order"));
        }
        if load_elf_header.e_ident[6] != 1 {
            return Err(anyhow!("Only the current ELF version is supported"));
        }
        let elf_header = Elf64Header::new(load_elf_header);
        let endian: Endian = elf_header.e_ident.endian();

        let phnum = elf_header.e_phnum.value(&endian);
        let shnum = elf_header.e_shnum.value(&endian);
        let shstrndx = elf_header.e_shstrndx.value(&endian);
        if phnum == 0xffff || shstrndx == 0xffff {
            return Err(anyhow!("ELF64 extended header numbering is not supported"));
        }
        if shnum == 0 && elf_header.e_shoff.value(&endian) != 0 {
            return Err(anyhow!("ELF64 extended section counts are not supported"));
        }

        let program_range = table_range(
            elf_header.e_phoff.value(&endian),
            phnum,
            elf_header.e_phentsize.value(&endian),
            buf.len(),
            "Program header table",
        )?;
        let section_range = table_range(
            elf_header.e_shoff.value(&endian),
            shnum,
            elf_header.e_shentsize.value(&endian),
            buf.len(),
            "Section header table",
        )?;
        if let (Some(program), Some(section)) = (&program_range, &section_range)
            && program.start < section.end
            && section.start < program.end
        {
            return Err(anyhow!("Program and section header tables overlap"));
        }

        let program_headers = parse_program_headers(buf, &elf_header, &endian)?;
        let section_headers = parse_section_headers(buf, &elf_header, &endian)?;

        Ok(Self {
            header: elf_header,
            program_headers,
            section_headers,
            raw: Cow::Borrowed(buf),
        })
    }

    pub fn strtab(&'a self) -> Result<&'a [u8]> {
        let endian = &self.endian();
        let strtab_section_index = self.header.e_shstrndx.value(endian) as usize;
        let strtab_section = self
            .section_headers
            .get(strtab_section_index)
            .context("String table section index is out of bounds")?;

        let strtab_section_offset = usize::try_from(strtab_section.sh_offset.value(endian))
            .context("strtab offset does not fit in usize")?;
        let strtab_section_size = usize::try_from(strtab_section.sh_size.value(endian))
            .context("strtab size does not fit in usize")?;
        let strtab_end = strtab_section_offset
            .checked_add(strtab_section_size)
            .context("String table range overflow")?;

        self.raw
            .get(strtab_section_offset..strtab_end)
            .ok_or(anyhow!("String table is outside the file bounds"))
    }

    pub fn resolve_section_name(
        &self,
        section: &Elf64SectionHeader,
        endian: &Endian,
    ) -> Result<&str> {
        let strtab_section_index = self.header.e_shstrndx.value(endian) as usize;
        let strtab_section = self
            .section_headers
            .get(strtab_section_index)
            .context("String table section index is out of bounds")?;

        let strtab_section_offset = usize::try_from(strtab_section.sh_offset.value(endian))
            .context("strtab offset does not fit in usize")?;
        let sh_name_index = usize::try_from(section.sh_name.value(endian))
            .context("Section name index does not fit in usize")?;

        let strtab_section_size = usize::try_from(strtab_section.sh_size.value(endian))
            .context("strtab size does not fit in usize")?;
        if sh_name_index >= strtab_section_size {
            return Err(anyhow!("Section name offset is outside the string table"));
        }
        let start = strtab_section_offset
            .checked_add(sh_name_index)
            .context("Section name offset overflow")?;
        let strtab_end = strtab_section_offset
            .checked_add(strtab_section_size)
            .context("String table range overflow")?;

        let raw_name = &self
            .raw
            .get(start..strtab_end)
            .context("Section name offset is outside the file bounds")?;

        let name = read_cstring(raw_name).context("Invalid section name")?;
        Ok(name)
    }

    pub fn endian(&self) -> Endian {
        self.header.e_ident.endian()
    }

    pub(crate) fn ensure_x86_64_little_endian(&self, operation: &str) -> Result<()> {
        let endian = self.endian();
        let machine = endian.read_u16(*self.header.e_machine.raw);
        if !matches!(endian, Endian::Little) || machine != 62 {
            return Err(anyhow!(
                "{operation} supports only little-endian x86-64 ELF files"
            ));
        }
        Ok(())
    }

    pub fn disasm(&'a self, dto: DisasmDTO<'a>) -> DisasmBinary<'a> {
        DisasmBinary { binary: self, dto }
    }

    pub fn update(&'a mut self, dto: UpdateDTO<'a>) -> UpdateBinary<'a> {
        UpdateBinary { binary: self, dto }
    }

    pub fn info(&'a self, dto: InfoDTO<'a>) -> InfoBinary<'a> {
        InfoBinary { binary: self, dto }
    }

    pub fn check_inject(&'a self, dto: CheckInjectDTO<'a>) -> CheckInjectBinary<'a> {
        CheckInjectBinary { binary: self, dto }
    }

    pub fn inject(&'a mut self, dto: InjectDTO<'a>) -> InjectBinary<'a> {
        InjectBinary { binary: self, dto }
    }

    pub fn entry(&self) -> u64 {
        let endian = self.endian();
        endian.read_u64(*self.header.e_entry.raw)
    }

    pub fn get_address_to_inject(&self) -> Result<u64> {
        let program_headers = &self.program_headers;
        let endian = self.endian();
        let mut higher_addr: u64 = 0;
        for program in program_headers {
            let initial_address = endian.read_u64(*program.p_vaddr.raw);
            let memsz = max(
                endian.read_u64(*program.p_memsz.raw),
                endian.read_u64(*program.p_filesz.raw),
            );
            let final_address = initial_address
                .checked_add(memsz)
                .context("Program memory range overflow")?;
            if final_address > higher_addr {
                higher_addr = final_address;
            }
        }
        let candidate = higher_addr
            .checked_add(ALIGN)
            .context("Injection address overflow")?;
        self.calculate_new_addr(candidate)
    }

    pub fn calculate_new_addr(&self, addr: u64) -> Result<u64> {
        let bytes: Vec<u8> = self
            .try_into()
            .context("Failed to convert binary into raw bytes")?;
        let offset = u64::try_from(bytes.len()).context("Binary too large to fit into u64")?;
        let delta = (offset % ALIGN + ALIGN - (addr % ALIGN)) % ALIGN;
        addr.checked_add(delta).context("Aligned address overflow")
    }
}

impl<'a> Binary for Elf64Binary<'a> {
    type Header = Elf64Header<'a>;
    type ProgramHeader = Elf64ProgramHeader<'a>;
    type SectionHeader = Elf64SectionHeader<'a>;

    fn get_header(&self) -> &Self::Header {
        &self.header
    }

    fn get_program_headers(&self) -> &[Self::ProgramHeader] {
        &self.program_headers
    }

    fn get_section_headers(&self) -> &[Self::SectionHeader] {
        &self.section_headers
    }
}

impl<'a> TryFrom<&'a Elf64Binary<'a>> for Vec<u8> {
    type Error = anyhow::Error;

    #[allow(clippy::too_many_lines)]
    fn try_from(h: &'a Elf64Binary<'a>) -> Result<Self, Self::Error> {
        let mut bytes: Vec<u8> = Vec::with_capacity(h.raw.len());
        let endian = &h.endian();

        let mut current_offset: usize = 0;

        let header_bytes: Vec<u8> = (&h.header).into();
        bytes.resize(header_bytes.len(), 0);
        bytes[current_offset..header_bytes.len()].copy_from_slice(&header_bytes);

        current_offset = header_bytes.len();

        let phoff = usize::try_from(h.header.e_phoff.value(endian))
            .context("invalid e_phoff: program header table offset does not fit usize")?;
        let shoff = usize::try_from(h.header.e_shoff.value(endian))
            .context("invalid e_shoff: section header table offset does not fit usize")?;

        let ph_first = phoff < shoff;

        if ph_first {
            if current_offset < phoff {
                let slice = &h
                    .raw
                    .get(current_offset..phoff)
                    .context("raw ELF image truncated before program headers")?;
                let new_len = current_offset
                    .checked_add(slice.len())
                    .context("offset overflow while expanding bytes for inter-header gap")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = phoff;
            }

            for ph in h.get_program_headers() {
                let ph_bytes: Vec<u8> = ph.into();
                let new_len = current_offset
                    .checked_add(ph_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&ph_bytes);
                current_offset = new_len;
            }

            if current_offset < shoff {
                let slice = h
                    .raw
                    .get(current_offset..shoff)
                    .context("raw ELF does not contain padding before section header table")?;
                let new_len = current_offset
                    .checked_add(slice.len())
                    .context("offset overflow copying section header entry")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = shoff;
            }

            for sh in h.get_section_headers() {
                let sh_bytes: Vec<u8> = sh.into();
                let new_len = current_offset
                    .checked_add(sh_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&sh_bytes);
                current_offset = new_len;
            }
        } else {
            if current_offset < shoff {
                let slice = &h
                    .raw
                    .get(current_offset..shoff)
                    .context("raw ELF does not contain padding before section header table")?;
                let new_len = current_offset
                    .checked_add(slice.len())
                    .context("offset overflow copying section header entry")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = shoff;
            }

            for sh in h.get_section_headers() {
                let sh_bytes: Vec<u8> = sh.into();
                let new_len = current_offset
                    .checked_add(sh_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&sh_bytes);
                current_offset = new_len;
            }

            if current_offset < phoff {
                let slice = h
                    .raw
                    .get(current_offset..phoff)
                    .context("raw ELF image truncated before program headers")?;
                let new_len = current_offset
                    .checked_add(slice.len())
                    .context("offset overflow while expanding bytes for inter-header gap")?;
                bytes.resize(new_len, 0);
                bytes[current_offset..new_len].copy_from_slice(slice);
                current_offset = phoff;
            }

            for ph in h.get_program_headers() {
                let ph_bytes: Vec<u8> = ph.into();
                let new_len = current_offset
                    .checked_add(ph_bytes.len())
                    .context("overflow copying section header")?;
                if bytes.len() < new_len {
                    bytes.resize(new_len, 0);
                }
                bytes[current_offset..new_len].copy_from_slice(&ph_bytes);
                current_offset = new_len;
            }
        }

        let slice = h.raw.get(current_offset..).context("Invalid offset")?;
        let new_len = current_offset
            .checked_add(slice.len())
            .context("Overflow new len")?;
        if bytes.len() < new_len {
            bytes.resize(new_len, 0);
        }
        bytes[current_offset..].copy_from_slice(slice);

        Ok(bytes)
    }
}

impl<'a> TryFrom<&'a mut Elf64Binary<'a>> for Vec<u8> {
    type Error = anyhow::Error;

    fn try_from(binary: &'a mut Elf64Binary<'a>) -> Result<Self, Self::Error> {
        Self::try_from(&*binary)
    }
}

#[cfg(test)]
mod tests {
    use super::{Elf64Binary, calculate_rel32};

    fn minimal_elf64_header() -> [u8; 64] {
        let mut bytes = [0_u8; 64];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[6] = 1;
        bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
        bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
        bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
        bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
        bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
        bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());
        bytes
    }

    #[test]
    fn calculates_positive_and_negative_rel32_offsets() {
        assert_eq!(calculate_rel32(0x1000, 0x1010).unwrap(), 0x10);
        assert_eq!(calculate_rel32(0x1010, 0x1000).unwrap(), -0x10);
    }

    #[test]
    fn rejects_offsets_outside_rel32_range() {
        assert!(calculate_rel32(0, i32::MAX as u64 + 1).is_err());
        assert!(calculate_rel32(i32::MAX as u64 + 2, 0).is_err());
    }

    #[test]
    fn rejects_a_truncated_elf_header() {
        assert!(Elf64Binary::new(&[0_u8; 63]).is_err());
    }

    #[test]
    fn rejects_invalid_elf_identification() {
        let mut raw = minimal_elf64_header();
        raw[0] = 0;
        assert!(Elf64Binary::new(&raw).is_err());

        let mut raw = minimal_elf64_header();
        raw[4] = 1;
        assert!(Elf64Binary::new(&raw).is_err());

        let mut raw = minimal_elf64_header();
        raw[5] = 0;
        assert!(Elf64Binary::new(&raw).is_err());

        let mut raw = minimal_elf64_header();
        raw[6] = 0;
        assert!(Elf64Binary::new(&raw).is_err());
    }

    #[test]
    fn rejects_extended_header_numbering() {
        let mut raw = minimal_elf64_header();
        raw[56..58].copy_from_slice(&0xffff_u16.to_le_bytes());
        assert!(Elf64Binary::new(&raw).is_err());

        let mut raw = minimal_elf64_header();
        raw[62..64].copy_from_slice(&0xffff_u16.to_le_bytes());
        assert!(Elf64Binary::new(&raw).is_err());

        let mut raw = minimal_elf64_header();
        raw[40..48].copy_from_slice(&64_u64.to_le_bytes());
        assert!(Elf64Binary::new(&raw).is_err());
    }

    #[test]
    fn rejects_truncated_program_header_table() {
        let mut raw = minimal_elf64_header();
        raw[32..40].copy_from_slice(&64_u64.to_le_bytes());
        raw[56..58].copy_from_slice(&1_u16.to_le_bytes());

        assert!(Elf64Binary::new(&raw).is_err());
    }

    #[test]
    fn rejects_truncated_section_header_table() {
        let mut raw = minimal_elf64_header();
        raw[40..48].copy_from_slice(&64_u64.to_le_bytes());
        raw[60..62].copy_from_slice(&1_u16.to_le_bytes());

        assert!(Elf64Binary::new(&raw).is_err());
    }

    #[test]
    fn rejects_header_tables_that_overlap_the_elf_header() {
        let mut raw = [minimal_elf64_header().as_slice(), &[0_u8; 56]].concat();
        raw[32..40].copy_from_slice(&32_u64.to_le_bytes());
        raw[56..58].copy_from_slice(&1_u16.to_le_bytes());

        assert!(Elf64Binary::new(&raw).is_err());
    }

    #[test]
    fn rejects_overlapping_program_and_section_header_tables() {
        let mut raw = vec![0_u8; 184];
        raw[..64].copy_from_slice(&minimal_elf64_header());
        raw[32..40].copy_from_slice(&64_u64.to_le_bytes());
        raw[40..48].copy_from_slice(&100_u64.to_le_bytes());
        raw[56..58].copy_from_slice(&1_u16.to_le_bytes());
        raw[60..62].copy_from_slice(&1_u16.to_le_bytes());

        assert!(Elf64Binary::new(&raw).is_err());
    }

    #[test]
    fn parses_and_serializes_a_minimal_elf64_header() {
        let raw = minimal_elf64_header();
        let binary = Elf64Binary::new(&raw).unwrap();
        let serialized = Vec::<u8>::try_from(&binary).unwrap();

        assert_eq!(serialized, raw);
        assert_eq!(binary.entry(), 0);
    }
}
