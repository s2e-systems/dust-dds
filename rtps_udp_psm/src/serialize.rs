use std::io::Write;
use byteorder::ByteOrder;

pub type Result = std::result::Result<(), std::io::Error>;

pub trait Serialize {
    fn serialize<W: Write, B: ByteOrder>(self, writer: W) -> Result;
}

