use binkit::elf64::{Elf64Binary, calculate_rel32};

fn minimal_elf64_header() -> [u8; 64] {
    let mut bytes = [0_u8; 64];
    bytes[0..4].copy_from_slice(b"\x7fELF");
    bytes[4] = 2;
    bytes[5] = 1;
    bytes[6] = 1;
    bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
    bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
    bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
    bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
    bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
    bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());
    bytes
}

#[test]
fn public_api_parses_and_serializes_elf64() {
    let raw = minimal_elf64_header();
    let binary = Elf64Binary::new(&raw).expect("minimal ELF64 header should parse");
    let serialized = Vec::<u8>::try_from(&binary).expect("ELF64 should serialize");

    assert_eq!(serialized, raw);
    assert_eq!(calculate_rel32(0x1000, 0x1010).unwrap(), 0x10);
}
