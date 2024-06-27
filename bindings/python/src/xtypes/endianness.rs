#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdrEndianness {
    LittleEndian,
    BigEndian,
}

pub type RepresentationIdentifier = [u8; 2];
pub type RepresentationOptions = [u8; 2];
pub const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
pub const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
pub const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];