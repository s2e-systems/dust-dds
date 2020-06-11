use std::convert::TryInto;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsSerdesResult, Endianness, };

pub type Long = i32;

impl RtpsSerialize for Long {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
        let value = match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        };
        writer.write(&value)?;
        Ok(())
    }
}

impl RtpsDeserialize for Long {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> { 
        let value = match endianness {
            Endianness::BigEndian => i32::from_be_bytes(bytes[0..4].try_into()?),
            Endianness::LittleEndian => i32::from_le_bytes(bytes[0..4].try_into()?),
        };
        Ok(value)
    }
}


pub type ULong = u32;

impl RtpsSerialize for ULong {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()> {
        let value = match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        };
        writer.write(&value)?;
        Ok(())
    }
}

impl RtpsDeserialize for ULong {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> { 
        let value = match endianness {
            Endianness::BigEndian => u32::from_be_bytes(bytes[0..4].try_into()?),
            Endianness::LittleEndian => u32::from_le_bytes(bytes[0..4].try_into()?),
        };
        Ok(value)
    }
}



pub type Short = i16;

impl RtpsSerialize for Short {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
        let value = match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        };
        writer.write(&value)?;
        Ok(())
    }
}

impl RtpsDeserialize for Short {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> { 
        let value = match endianness {
            Endianness::BigEndian => i16::from_be_bytes(bytes[0..2].try_into()?),
            Endianness::LittleEndian => i16::from_le_bytes(bytes[0..2].try_into()?),
        };
        Ok(value)
    }
}



pub type UShort = u16;

impl RtpsSerialize for UShort {
    fn serialize(&self, writer: &mut impl std::io::Write, endianness: Endianness) -> RtpsSerdesResult<()>{
        let value = match endianness {
            Endianness::BigEndian => self.to_be_bytes(),
            Endianness::LittleEndian => self.to_le_bytes(),
        };
        writer.write(&value)?;
        Ok(())
    }
}

impl RtpsDeserialize for UShort {
    fn deserialize(bytes: &[u8], endianness: Endianness) -> RtpsSerdesResult<Self> { 
        let value = match endianness {
            Endianness::BigEndian => u16::from_be_bytes(bytes[0..2].try_into()?),
            Endianness::LittleEndian => u16::from_le_bytes(bytes[0..2].try_into()?),
        };
        Ok(value)
    }
}
