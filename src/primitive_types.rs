use std::convert::TryInto;
use crate::serdes::{PrimitiveSerdes, RtpsSerialize, RtpsDeserialize, RtpsSerdesResult, EndianessFlag, };


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Long(pub i32);

impl RtpsSerialize for Long
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        writer.write(&PrimitiveSerdes::serialize_i32(self.0, endianness))?;
        Ok(())
    }
}

impl From<Long> for usize {
    fn from(value: Long) -> Self {
        value.0 as usize
    }
}

impl RtpsDeserialize for Long {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_i32(bytes[0..4].try_into()?, endianness);
        Ok(Self(value))
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct ULong(pub u32);

impl RtpsSerialize for ULong {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()> {
        writer.write(&PrimitiveSerdes::serialize_u32(self.0, endianness))?;
        Ok(())
    }
}

impl From<ULong> for usize {
    fn from(value: ULong) -> Self {
        value.0 as usize
    }
}

impl From<usize> for ULong {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl RtpsDeserialize for ULong {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_u32(bytes[0..4].try_into()?, endianness);
        Ok(Self(value))
    }
}



#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub struct Short(pub i16);

impl RtpsSerialize for Short
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        let value = self.0;
        writer.write(&PrimitiveSerdes::serialize_i16(value, endianness))?;
        Ok(())
    }
}

impl From<Short> for usize {
    fn from(value: Short) -> Self {
        value.0 as usize
    }
}

impl From<usize> for Short {
    fn from(value: usize) -> Self {
        Self(value as i16)
    }    
}

impl RtpsDeserialize for Short {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_i16(bytes[0..2].try_into()?, endianness);
        Ok(Short(value))
    }
}



#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ushort(pub u16);

impl RtpsSerialize for Ushort
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        let value = self.0;
        writer.write(&PrimitiveSerdes::serialize_u16(value, endianness))?;
        Ok(())
    }
}

impl From<Ushort> for usize {
    fn from(value: Ushort) -> Self {
        value.0 as usize
    }
}

impl From<usize> for Ushort {
    fn from(value: usize) -> Self {
        Self(value as u16)
    }    
}

impl RtpsDeserialize for Ushort {
    fn deserialize(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self> { 
        let value = PrimitiveSerdes::deserialize_u16(bytes[0..2].try_into()?, endianness);
        Ok(Ushort(value))
    }
}
