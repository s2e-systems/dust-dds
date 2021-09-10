use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub const ENCAPSULATION_HEADER_SIZE: u64 = 4;

/// Data encapsulation scheme identifiers.
pub trait Encapsulation {
    type E: ByteOrder;

    fn id() -> [u8; 2];
    fn option() -> [u8; 2] {
        [0; 2]
    }
}

/// OMG CDR big-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CdrBe {}

impl Encapsulation for CdrBe {
    type E = BigEndian;

    fn id() -> [u8; 2] {
        [0, 0]
    }
}

/// OMG CDR little-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CdrLe {}

impl Encapsulation for CdrLe {
    type E = LittleEndian;

    fn id() -> [u8; 2] {
        [0, 1]
    }
}

/// ParameterList encapsulated using OMG CDR big-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlCdrBe {}

impl Encapsulation for PlCdrBe {
    type E = BigEndian;

    fn id() -> [u8; 2] {
        [0, 2]
    }
}

/// ParameterList encapsulated using OMG CDR little-endian encapsulation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlCdrLe {}

impl Encapsulation for PlCdrLe {
    type E = LittleEndian;

    fn id() -> [u8; 2] {
        [0, 3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (CdrBe::id().len() + CdrBe::option().len()) as u64
        );
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (CdrLe::id().len() + CdrLe::option().len()) as u64
        );
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (PlCdrBe::id().len() + PlCdrBe::option().len()) as u64
        );
        assert_eq!(
            ENCAPSULATION_HEADER_SIZE,
            (PlCdrLe::id().len() + PlCdrLe::option().len()) as u64
        );
    }
}
