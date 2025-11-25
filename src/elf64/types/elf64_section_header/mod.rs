mod sh_addr;
mod sh_addralign;
mod sh_entsize;
mod sh_flags;
mod sh_info;
mod sh_link;
mod sh_name;
mod sh_offset;
mod sh_size;
mod sh_type;

use std::borrow::Cow;

use sh_addr::ShAddr;
use sh_addralign::ShAddralign;
use sh_entsize::ShEntsize;
use sh_flags::ShFlags;
use sh_info::ShInfo;
use sh_link::ShLink;
use sh_name::ShName;
use sh_offset::ShOffset;
use sh_size::ShSize;
use sh_type::ShType;

use crate::elf64::loaders::load_elf64_section_header::LoadELF64SectionHeader;

#[derive(Debug)]
pub struct Elf64SectionHeader<'a> {
    pub sh_name: ShName<'a>,
    pub sh_type: ShType<'a>,
    pub sh_flags: ShFlags<'a>,
    pub sh_addr: ShAddr<'a>,
    pub sh_offset: ShOffset<'a>,
    pub sh_size: ShSize<'a>,
    pub sh_link: ShLink<'a>,
    pub sh_info: ShInfo<'a>,
    pub sh_addralign: ShAddralign<'a>,
    pub sh_entsize: ShEntsize<'a>
}

impl<'a> Elf64SectionHeader<'a> {
    pub fn new(load: &'a LoadELF64SectionHeader) -> Self {
        Self {
            sh_name: ShName::new(Cow::Borrowed(&load.sh_name)),
            sh_type: ShType::new(Cow::Borrowed(&load.sh_type)),
            sh_flags: ShFlags::new(Cow::Borrowed(&load.sh_flags)),
            sh_addr: ShAddr::new(Cow::Borrowed(&load.sh_addr)),
            sh_offset: ShOffset::new(Cow::Borrowed(&load.sh_offset)),
            sh_size: ShSize::new(Cow::Borrowed(&load.sh_size)),
            sh_link: ShLink::new(Cow::Borrowed(&load.sh_link)),
            sh_info: ShInfo::new(Cow::Borrowed(&load.sh_info)),
            sh_addralign: ShAddralign::new(Cow::Borrowed(&load.sh_addralign)),
            sh_entsize: ShEntsize::new(Cow::Borrowed(&load.sh_entsize)),
        }
    }
}

impl<'a> From<&Elf64SectionHeader<'a>> for Vec<u8> {
    fn from(h: &Elf64SectionHeader) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&*h.sh_name.raw);
        bytes.extend_from_slice(&*h.sh_type.raw);
        bytes.extend_from_slice(&*h.sh_flags.raw);
        bytes.extend_from_slice(&*h.sh_addr.raw);
        bytes.extend_from_slice(&*h.sh_offset.raw);
        bytes.extend_from_slice(&*h.sh_size.raw);
        bytes.extend_from_slice(&*h.sh_link.raw);
        bytes.extend_from_slice(&*h.sh_info.raw);
        bytes.extend_from_slice(&*h.sh_addralign.raw);
        bytes.extend_from_slice(&*h.sh_entsize.raw);
        bytes
    }
}
