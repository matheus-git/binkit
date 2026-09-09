use binkit::Endian;
use binkit::elf64::{Elf64Binary, calculate_rel32};
use proptest::prelude::*;

fn elf64_header_with_entry(entry: u64) -> [u8; 64] {
    let mut bytes = [0_u8; 64];
    bytes[0..4].copy_from_slice(b"\x7fELF");
    bytes[4] = 2;
    bytes[5] = 1;
    bytes[6] = 1;
    bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
    bytes[18..20].copy_from_slice(&62_u16.to_le_bytes());
    bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
    bytes[24..32].copy_from_slice(&entry.to_le_bytes());
    bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
    bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
    bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());
    bytes
}

proptest! {
    #[test]
    fn endian_u64_round_trip(value in any::<u64>()) {
        prop_assert_eq!(Endian::Little.read_u64(Endian::Little.to_bytes_u64(value)), value);
        prop_assert_eq!(Endian::Big.read_u64(Endian::Big.to_bytes_u64(value)), value);
    }

    #[test]
    fn valid_rel32_difference_round_trips(base in any::<u64>(), difference in any::<i32>()) {
        let target = i128::from(base) + i128::from(difference);
        prop_assume!((0..=i128::from(u64::MAX)).contains(&target));
        prop_assert_eq!(calculate_rel32(base, target as u64).unwrap(), difference);
    }

    #[test]
    fn minimal_elf_parse_serialize_parse_preserves_entry(entry in any::<u64>()) {
        let raw = elf64_header_with_entry(entry);
        let first = Elf64Binary::new(&raw).unwrap();
        let serialized = Vec::<u8>::try_from(&first).unwrap();
        let second = Elf64Binary::new(&serialized).unwrap();

        prop_assert_eq!(serialized.as_slice(), raw.as_slice());
        prop_assert_eq!(second.entry(), entry);
    }
}
