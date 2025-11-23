pub struct UpdateDTO<'a> {
    pub file:&'a str,
    pub entry: Option<&'a str>,
    pub output: Option<&'a str>
}
