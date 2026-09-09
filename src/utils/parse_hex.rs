use anyhow::{Context, Result, anyhow};

pub fn parse_hex_to_u64(s: &str) -> Result<u64> {
    let s = s.trim();
    let hex = if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        rest
    } else {
        s
    };

    if hex.is_empty() {
        return Err(anyhow!("invalid hex: empty string"));
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("invalid hex: contains non-hex characters: {}", s));
    }

    let result = u64::from_str_radix(hex, 16).context("invalid hex: value does not fit in u64")?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::parse_hex_to_u64;

    #[test]
    fn parses_lowercase_prefix() {
        assert_eq!(parse_hex_to_u64("0x2a").unwrap(), 42);
    }

    #[test]
    fn parses_uppercase_prefix() {
        assert_eq!(parse_hex_to_u64("0X2A").unwrap(), 42);
    }

    #[test]
    fn parses_value_without_prefix() {
        assert_eq!(parse_hex_to_u64("2A").unwrap(), 42);
    }

    #[test]
    fn rejects_empty_value() {
        assert!(parse_hex_to_u64("").is_err());
        assert!(parse_hex_to_u64("0x").is_err());
    }

    #[test]
    fn rejects_non_hex_characters() {
        assert!(parse_hex_to_u64("0x2g").is_err());
    }

    #[test]
    fn rejects_u64_overflow() {
        assert!(parse_hex_to_u64("10000000000000000").is_err());
    }
}
