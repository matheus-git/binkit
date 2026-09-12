use std::fmt::Write;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().saturating_mul(3).saturating_sub(1));
    for (index, byte) in bytes.iter().enumerate() {
        if index != 0 {
            output.push(' ');
        }
        write!(output, "{byte:02X}").expect("writing to a String cannot fail");
    }
    output
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
