pub struct DisasmDTO<'a> {
    pub file: &'a str,
    pub section: Option<&'a str>,
    pub offset: Option<usize>,
    pub address: Option<&'a str>,
    pub count: Option<usize>,
    pub bytes: Option<usize>,
}
