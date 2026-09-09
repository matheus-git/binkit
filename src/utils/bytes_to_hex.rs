pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::bytes_to_hex;

    #[test]
    fn formats_bytes_as_uppercase_hex() {
        assert_eq!(bytes_to_hex(&[0x00, 0x0a, 0xff]), "00 0A FF");
    }

    #[test]
    fn formats_empty_slice_as_empty_string() {
        assert_eq!(bytes_to_hex(&[]), "");
    }
}
