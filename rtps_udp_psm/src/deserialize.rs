use byteorder::{ByteOrder, LittleEndian};

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait Deserialize<'de> : Sized{
    fn deserialize<B>(buf: &mut &'de[u8]) -> Result<Self> where B: ByteOrder;
}

pub fn from_bytes_le<'de, D: Deserialize<'de>>(mut buf: &'de [u8]) -> Result<D> {
    D::deserialize::<LittleEndian>(&mut buf)
}