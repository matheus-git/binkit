pub fn parse_hex_to_u64(s: &str) -> u64 {
    let s = s.trim();
    let hex = if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        rest
    } else {
        s
    };

    if hex.is_empty() {
        panic!("invalid hex: empty string");
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        panic!("invalid hex: contains non-hex characters: {}", s);
    }

    u64::from_str_radix(hex, 16).unwrap_or_else(|err| {
        panic!("failed to parse hex `{}` into u64: {}", s, err);
    })
}
