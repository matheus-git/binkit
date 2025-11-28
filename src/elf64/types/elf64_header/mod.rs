mod e_ident;
mod e_type;
mod e_machine;
mod e_version;
mod e_entry;
mod e_phoff;
mod e_shoff;
mod e_flags;
mod e_ehsize;
mod e_phentsize;
mod e_phnum;
mod e_shentsize;
mod e_shnum;
mod e_shstrndx;

use std::borrow::Cow;

use e_ehsize::EEhsize;
use e_entry::EEntry;
use e_flags::EFlags;
use e_ident::EIdent;
use e_machine::EMachine;
use e_phentsize::EPhentsize;
use e_phnum::EPhnum;
use e_phoff::EPhoff;
use e_shentsize::EShentsize;
use e_shnum::EShnum;
use e_shoff::EShoff;
use e_shstrndx::EShstrndx;
use e_type::EType;
use e_version::EVersion;

use crate::elf64::loaders::load_elf64_header::LoadELF64Header;

#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct Elf64Header<'a> {
    pub e_ident: EIdent<'a>,
    pub e_type: EType<'a>,
    pub e_machine: EMachine<'a>,
    pub e_version: EVersion<'a>,
    pub e_entry: EEntry<'a>,
    pub e_phoff: EPhoff<'a>,
    pub e_shoff: EShoff<'a>,
    pub e_flags: EFlags<'a>,
    pub e_ehsize: EEhsize<'a>,
    pub e_phentsize: EPhentsize<'a>,
    pub e_phnum: EPhnum<'a>,
    pub e_shentsize: EShentsize<'a>,
    pub e_shnum: EShnum<'a>,
    pub e_shstrndx: EShstrndx<'a>
}

impl<'a> Elf64Header<'a> {
    pub fn new(load: &'a LoadELF64Header) -> Self {
        Self { 
            e_ident: EIdent::new(Cow::Borrowed(&load.e_ident)),
            e_type: EType::new(Cow::Borrowed(&load.e_type)),
            e_machine: EMachine::new(Cow::Borrowed(&load.e_machine)),
            e_version: EVersion::new(Cow::Borrowed(&load.e_version)),
            e_entry: EEntry::new(Cow::Borrowed(&load.e_entry)),
            e_phoff: EPhoff::new(Cow::Borrowed(&load.e_phoff)),
            e_shoff: EShoff::new(Cow::Borrowed(&load.e_shoff)),
            e_flags: EFlags::new(Cow::Borrowed(&load.e_flags)),
            e_ehsize: EEhsize::new(Cow::Borrowed(&load.e_ehsize)),
            e_phentsize: EPhentsize::new(Cow::Borrowed(&load.e_phentsize)),
            e_phnum: EPhnum::new(Cow::Borrowed(&load.e_phnum)),
            e_shentsize: EShentsize::new(Cow::Borrowed(&load.e_shentsize)),
            e_shnum: EShnum::new(Cow::Borrowed(&load.e_shnum)),
            e_shstrndx: EShstrndx::new(Cow::Borrowed(&load.e_shstrndx))
        }
    }
}

impl<'a> From<&Elf64Header<'a>> for Vec<u8> {
    fn from(h: &Elf64Header<'a>) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&*h.e_ident.raw);
        bytes.extend_from_slice(&*h.e_type.raw);
        bytes.extend_from_slice(&*h.e_machine.raw);
        bytes.extend_from_slice(&*h.e_version.raw);
        bytes.extend_from_slice(&*h.e_entry.raw);
        bytes.extend_from_slice(&*h.e_phoff.raw);
        bytes.extend_from_slice(&*h.e_shoff.raw);
        bytes.extend_from_slice(&*h.e_flags.raw);
        bytes.extend_from_slice(&*h.e_ehsize.raw);
        bytes.extend_from_slice(&*h.e_phentsize.raw);
        bytes.extend_from_slice(&*h.e_phnum.raw);
        bytes.extend_from_slice(&*h.e_shentsize.raw);
        bytes.extend_from_slice(&*h.e_shnum.raw);
        bytes.extend_from_slice(&*h.e_shstrndx.raw);
        bytes
    }
}
