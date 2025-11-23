pub struct CheckInjectDTO<'a> {
    pub file: &'a str,
    pub return_address: Option<&'a str>
}
