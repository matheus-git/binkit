mod p_type;
mod p_paddr;
mod p_flags;
mod p_align;
mod p_filesz;
mod p_memsz;
mod p_offset;
mod p_vaddr;

use std::borrow::Cow;

use p_align::PAlign;
use p_type::PType;
use p_paddr::PPaddr;
use p_flags::PFlags;
use p_filesz::PFilesz;
use p_memsz::PMemsz;
use p_offset::POffset;
use p_vaddr::PVaddr;
use super::super::LoadELF64ProgramHeader;

#[derive(Debug)]
pub struct Elf64ProgramHeader<'a> {
    pub p_type: PType<'a>,
    pub p_flags: PFlags<'a>,
    pub p_offset: POffset<'a>,
    pub p_vaddr: PVaddr<'a>,
    pub p_paddr: PPaddr<'a>,
    pub p_filesz: PFilesz<'a>,
    pub p_memsz: PMemsz<'a>,
    pub p_align: PAlign<'a>
}

impl<'a> Elf64ProgramHeader<'a> {
    pub fn new(load: &'a LoadELF64ProgramHeader) -> Self {
        Self { 
            p_type: PType::new(Cow::Borrowed(&load.p_type)),
            p_flags: PFlags::new(Cow::Borrowed(&load.p_flags)),
            p_offset: POffset::new(Cow::Borrowed(&load.p_offset)),
            p_vaddr: PVaddr::new(Cow::Borrowed(&load.p_vaddr)),
            p_paddr: PPaddr::new(Cow::Borrowed(&load.p_paddr)),
            p_filesz: PFilesz::new(Cow::Borrowed(&load.p_filesz)),
            p_memsz: PMemsz::new(Cow::Borrowed(&load.p_memsz)),
            p_align: PAlign::new(Cow::Borrowed(&load.p_align)),
        }
    }
}

impl<'a> From<&Elf64ProgramHeader<'a>> for Vec<u8> {
    fn from(h: &Elf64ProgramHeader) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&*h.p_type.raw);
        bytes.extend_from_slice(&*h.p_flags.raw);
        bytes.extend_from_slice(&*h.p_offset.raw);
        bytes.extend_from_slice(&*h.p_vaddr.raw);
        bytes.extend_from_slice(&*h.p_paddr.raw);
        bytes.extend_from_slice(&*h.p_filesz.raw);
        bytes.extend_from_slice(&*h.p_memsz.raw);
        bytes.extend_from_slice(&*h.p_align.raw);
        bytes
    }
}
