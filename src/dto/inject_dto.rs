pub struct InjectDTO<'a> {
    pub file: &'a str,
    pub inject: &'a str, 
    pub address: Option<&'a str>,
    pub section: Option<&'a str>,
    pub return_address: Option<&'a str>,
    pub output: &'a str
}
