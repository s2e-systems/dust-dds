use std::io::{Read, Write};

use byteorder::{ByteOrder, WriteBytesExt};
use rust_rtps_pim::{structure::types::GuidPrefix};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for GuidPrefix {
    fn serialize<W: Write, B: ByteOrder>(&self, writer: W) -> serialize::Result {
        writer.write(self)?;
        Ok(())
    }
}
impl<'de>  Deserialize<'de> for GuidPrefix {
    fn deserialize<B>(buf: &mut &'de[u8]) -> deserialize::Result<Self> where B: ByteOrder {
        let mut value: Self;
        buf.read_exact(value.as_mut())?;
        Ok(value)
    }
}