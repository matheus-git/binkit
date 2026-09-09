pub trait Binary {
    type Header;
    type ProgramHeader;
    type SectionHeader;

    fn get_header(&self) -> &Self::Header;
    fn get_program_headers(&self) -> &[Self::ProgramHeader];
    fn get_section_headers(&self) -> &[Self::SectionHeader];
}
