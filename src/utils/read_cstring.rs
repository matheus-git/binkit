use std::str::Utf8Error;

pub fn read_cstring(bytes: &[u8]) -> Result<&str, Utf8Error> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end])
}
