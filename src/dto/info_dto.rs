pub struct InfoDTO<'a> {
    pub file:&'a str,
    pub header: bool,
    pub programs: bool,
    pub sections: bool
}
