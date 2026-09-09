//pub enum BinaryType {
//    Elf64,
//}
//
//impl BinaryType {
//    pub fn from_bytes(bytes: &[u8]) -> Option<BinaryType> {
//        if bytes.len() < 5 {
//            return None;
//        }
//
//        if &bytes[0..4] == b"\x7FELF" {
//            match bytes[4] {
//                2 => Some(BinaryType::Elf64),
//                _ => None
//            }
//        } else {
//            None
//        }
//    }
//}
