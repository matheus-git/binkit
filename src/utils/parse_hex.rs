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

    let result = u64::from_str_radix(hex, 16)?;

    Ok(result)
}
