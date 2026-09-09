pub enum Endian {
    Little,
    Big,
}

impl Endian {
    pub fn read_u16(&self, bytes: [u8; 2]) -> u16 {
        match self {
            Endian::Little => u16::from_le_bytes(bytes),
            Endian::Big => u16::from_be_bytes(bytes),
        }
    }

    pub fn read_u32(&self, bytes: [u8; 4]) -> u32 {
        match self {
            Endian::Little => u32::from_le_bytes(bytes),
            Endian::Big => u32::from_be_bytes(bytes),
        }
    }

    pub fn read_u64(&self, bytes: [u8; 8]) -> u64 {
        match self {
            Endian::Little => u64::from_le_bytes(bytes),
            Endian::Big => u64::from_be_bytes(bytes),
        }
    }

    pub fn to_bytes_u16(&self, value: u16) -> [u8; 2] {
        match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        }
    }

    pub fn to_bytes_u32(&self, value: u32) -> [u8; 4] {
        match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        }
    }

    pub fn to_bytes_u64(&self, value: u64) -> [u8; 8] {
        match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        }
    }

    pub fn to_bytes_i32(&self, value: i32) -> [u8; 4] {
        match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        }
    }

    pub fn to_bytes_i64(&self, value: i64) -> [u8; 8] {
        match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Endian;

    #[test]
    fn reads_unsigned_values_in_both_byte_orders() {
        assert_eq!(Endian::Little.read_u16([0x34, 0x12]), 0x1234);
        assert_eq!(Endian::Big.read_u16([0x12, 0x34]), 0x1234);
        assert_eq!(
            Endian::Little.read_u32([0x78, 0x56, 0x34, 0x12]),
            0x1234_5678
        );
        assert_eq!(Endian::Big.read_u32([0x12, 0x34, 0x56, 0x78]), 0x1234_5678);
        assert_eq!(
            Endian::Little.read_u64([0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01]),
            0x0123_4567_89ab_cdef
        );
        assert_eq!(
            Endian::Big.read_u64([0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]),
            0x0123_4567_89ab_cdef
        );
    }

    #[test]
    fn writes_unsigned_values_in_both_byte_orders() {
        assert_eq!(Endian::Little.to_bytes_u16(0x1234), [0x34, 0x12]);
        assert_eq!(Endian::Big.to_bytes_u16(0x1234), [0x12, 0x34]);
        assert_eq!(
            Endian::Little.to_bytes_u32(0x1234_5678),
            [0x78, 0x56, 0x34, 0x12]
        );
        assert_eq!(
            Endian::Big.to_bytes_u32(0x1234_5678),
            [0x12, 0x34, 0x56, 0x78]
        );
    }

    #[test]
    fn writes_signed_values_without_losing_twos_complement() {
        assert_eq!(Endian::Little.to_bytes_i32(-2), [0xfe, 0xff, 0xff, 0xff]);
        assert_eq!(Endian::Big.to_bytes_i32(-2), [0xff, 0xff, 0xff, 0xfe]);
        assert_eq!(
            Endian::Little.to_bytes_i64(-2),
            [0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
        assert_eq!(
            Endian::Big.to_bytes_i64(-2),
            [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe]
        );
    }

    #[test]
    fn u64_write_round_trips_through_read() {
        let value = 0x0123_4567_89ab_cdef;
        for endian in [Endian::Little, Endian::Big] {
            assert_eq!(endian.read_u64(endian.to_bytes_u64(value)), value);
        }
    }
}
