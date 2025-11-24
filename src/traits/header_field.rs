use crate::utils::endian::Endian;

pub trait HeaderField {
    type Value;

    fn describe(&self, endian: &Endian) -> String;
    fn value(&self, endian: &Endian) -> Self::Value;
}
