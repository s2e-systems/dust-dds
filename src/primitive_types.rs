use std::convert::TryInto;
use crate::serdes::{PrimitiveSerdes, RtpsSerialize, RtpsDeserialize, RtpsSerdesResult, EndianessFlag, };

pub type Long = i32;

impl RtpsSerialize for Long
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        writer.write(&PrimitiveSerdes::serialize_i32(*self, endianness))?;
        Ok(())
    }
}

impl RtpsDeserialize for Long {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_i32(bytes[0..4].try_into()?, endianness);
        Ok(value)
    }
}


pub type ULong = u32;

impl RtpsSerialize for ULong {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        writer.write(&PrimitiveSerdes::serialize_u32(*self, endianness))?;
        Ok(())
    }
}

impl RtpsDeserialize for ULong {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_u32(bytes[0..4].try_into()?, endianness);
        Ok(value)
    }
}



pub type Short = i16;

impl RtpsSerialize for Short {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        writer.write(&PrimitiveSerdes::serialize_i16(*self, endianness))?;
        Ok(())
    }
}

impl RtpsDeserialize for Short {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_i16(bytes[0..2].try_into()?, endianness);
        Ok(value)
    }
}



pub type UShort = u16;

impl RtpsSerialize for UShort {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        writer.write(&PrimitiveSerdes::serialize_u16(*self, endianness))?;
        Ok(())
    }
}
impl RtpsDeserialize for UShort {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_u16(bytes[0..2].try_into()?, endianness);
        Ok(value)
    }
}
