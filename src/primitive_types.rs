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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serdes::RtpsSerdesError;

    #[test]
    fn serialize_deserialize_ushort(){
        let mut buf = Vec::new();

        let ushort_val: UShort = 123;

        ushort_val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [123, 0]);
        assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), ushort_val);
        buf.clear();

        ushort_val.serialize(&mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0, 123]);
        assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), ushort_val);
        buf.clear();


        let ushort_max: UShort = u16::MAX;

        ushort_max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF]);
        assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), ushort_max);
        buf.clear();

        ushort_max.serialize(&mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0xFF, 0xFF]);
        assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), ushort_max);
        buf.clear();

        let ushort_min: UShort = u16::MIN;

        ushort_min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00]);
        assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), ushort_min);
        buf.clear();

        ushort_min.serialize(&mut buf, Endianness::BigEndian).unwrap();
        assert_eq!(buf, [0x00, 0x00]);
        assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), ushort_min);
        buf.clear();
    }

    #[test]
    fn ushort_invalid_deserialize() {
        let buf: [u8; 1] = [1];
        let result = UShort::deserialize(&buf, Endianness::BigEndian);
        match result {
            Err(RtpsSerdesError::InvalidTypeConversion) => assert!(true),
            _ => assert!(false),
        }
    }
}
