use byteorder::{ByteOrder, LittleEndian};

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait MappingRead<'de>: Sized {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self>;
}

pub trait MappingReadByteOrdered<'de>: Sized {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: ByteOrder;
}

pub fn from_bytes_le<'de, D: MappingReadByteOrdered<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::mapping_read_byte_ordered::<LittleEndian>(&mut buf)
}

pub fn from_bytes<'de, D: MappingRead<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::mapping_read(&mut buf)
}
