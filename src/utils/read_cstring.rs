use std::str::Utf8Error;

pub fn read_cstring(bytes: &[u8]) -> Result<&str, Utf8Error> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end])
}

#[cfg(test)]
mod tests {
    use super::read_cstring;

    #[test]
    fn stops_at_first_nul_byte() {
        assert_eq!(read_cstring(b"hello\0ignored").unwrap(), "hello");
    }

    #[test]
    fn accepts_text_without_nul_terminator() {
        assert_eq!(read_cstring(b"hello").unwrap(), "hello");
    }

    #[test]
    fn rejects_invalid_utf8_before_terminator() {
        assert!(read_cstring(&[0xff, 0]).is_err());
    }

    #[test]
    fn ignores_invalid_utf8_after_terminator() {
        assert_eq!(read_cstring(&[b'a', 0, 0xff]).unwrap(), "a");
    }
}
