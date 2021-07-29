use byteorder::ByteOrder;

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait Deserialize<'de> : Sized{
    fn deserialize<T>(buf: &mut &'de[u8]) -> Result<Self> where T: ByteOrder;
}
