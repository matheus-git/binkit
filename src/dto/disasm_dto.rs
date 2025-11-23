pub struct DisasmDTO<'a> {
    pub file:&'a str,
    pub section: Option<&'a str>,
}
