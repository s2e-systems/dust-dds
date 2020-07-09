use std::convert::TryInto;
use std::{i16, i32, u32};
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsSerdesResult, Endianness, SizeCheck};

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
        bytes.check_size_equal(4)?;

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
        bytes.check_size_equal(4)?;

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
        bytes.check_size_equal(2)?;

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
        bytes.check_size_equal(2)?;

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

    // #[test]
    // fn serialize_deserialize_ushort(){
    //     let mut buf = Vec::new();

    //     let val: UShort = 123;

    //     val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [123, 0]);
    //     assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
    //     buf.clear();

    //     val.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0, 123]);
    //     assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
    //     buf.clear();


    //     let max: UShort = UShort::MAX;

    //     max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0xFF, 0xFF]);
    //     assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
    //     buf.clear();

    //     max.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0xFF, 0xFF]);
    //     assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
    //     buf.clear();

    //     let min: UShort = UShort::MIN;

    //     min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0x00, 0x00]);
    //     assert_eq!(UShort::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
    //     buf.clear();

    //     min.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0x00, 0x00]);
    //     assert_eq!(UShort::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
    //     buf.clear();
    // }

    #[test]
    fn invalid_ushort_deserialize() {
        let buf: [u8; 1] = [1];
        let result = UShort::deserialize(&buf, Endianness::BigEndian);
        match result {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }

    // #[test]
    // fn serialize_deserialize_short(){
    //     let mut buf = Vec::new();

    //     let val: Short = 123;

    //     val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [123, 0]);
    //     assert_eq!(Short::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
    //     buf.clear();

    //     val.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0, 123]);
    //     assert_eq!(Short::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
    //     buf.clear();


    //     let max: Short = Short::MAX;

    //     max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0xFF, 0x7F]);
    //     assert_eq!(Short::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
    //     buf.clear();

    //     max.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0x7F, 0xFF]);
    //     assert_eq!(Short::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
    //     buf.clear();

    //     let min: Short = Short::MIN;

    //     min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0x00, 0x80]);
    //     assert_eq!(Short::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
    //     buf.clear();

    //     min.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0x80, 0x00]);
    //     assert_eq!(Short::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
    //     buf.clear();
    // }

    #[test]
    fn invalid_short_deserialize() {
        let buf: [u8; 1] = [1];
        let result = Short::deserialize(&buf, Endianness::BigEndian);
        match result {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }

    // #[test]
    // fn serialize_deserialize_long(){
    //     let mut buf = Vec::new();

    //     let val: Long = 1230;

    //     val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0xCE, 0x04, 0, 0]);
    //     assert_eq!(Long::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
    //     buf.clear();

    //     val.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0, 0, 0x04, 0xCE]);
    //     assert_eq!(Long::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
    //     buf.clear();


    //     let max: Long = Long::MAX;

    //     max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0x7F]);
    //     assert_eq!(Long::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
    //     buf.clear();

    //     max.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0x7F, 0xFF, 0xFF, 0xFF]);
    //     assert_eq!(Long::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
    //     buf.clear();

    //     let min: Long = Long::MIN;

    //     min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0x00, 0x00, 0x00, 0x80]);
    //     assert_eq!(Long::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
    //     buf.clear();

    //     min.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0x80, 0x00, 0x00, 0x00]);
    //     assert_eq!(Long::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
    //     buf.clear();
    // }

    #[test]
    fn invalid_long_deserialize() {
        let buf: [u8; 3] = [1, 2, 3];
        let result = Long::deserialize(&buf, Endianness::BigEndian);
        match result {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }

    // #[test]
    // fn serialize_deserialize_ulong(){
    //     let mut buf = Vec::new();

    //     let val: ULong = 1230;

    //     val.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0xCE, 0x04, 0, 0]);
    //     assert_eq!(ULong::deserialize(&buf, Endianness::LittleEndian).unwrap(), val);
    //     buf.clear();

    //     val.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0, 0, 0x04, 0xCE]);
    //     assert_eq!(ULong::deserialize(&buf, Endianness::BigEndian).unwrap(), val);
    //     buf.clear();


    //     let max: ULong = ULong::MAX;

    //     max.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
    //     assert_eq!(ULong::deserialize(&buf, Endianness::LittleEndian).unwrap(), max);
    //     buf.clear();

    //     max.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
    //     assert_eq!(ULong::deserialize(&buf, Endianness::BigEndian).unwrap(), max);
    //     buf.clear();

    //     let min: ULong = ULong::MIN;

    //     min.serialize(&mut buf, Endianness::LittleEndian).unwrap();
    //     assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
    //     assert_eq!(ULong::deserialize(&buf, Endianness::LittleEndian).unwrap(), min);
    //     buf.clear();

    //     min.serialize(&mut buf, Endianness::BigEndian).unwrap();
    //     assert_eq!(buf, [0x00, 0x00, 0x00, 0x00]);
    //     assert_eq!(ULong::deserialize(&buf, Endianness::BigEndian).unwrap(), min);
    //     buf.clear();
    // }

    #[test]
    fn invalid_ulong_deserialize() {
        let buf: [u8; 3] = [1, 2, 3];
        let result = ULong::deserialize(&buf, Endianness::BigEndian);
        match result {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        }
    }
}
