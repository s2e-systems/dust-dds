use std::io::{BufRead, Read};

use crate::serialized_payload::cdr::{deserialize::CdrDeserialize, deserializer::CdrDeserializer};

use super::endianness::CdrEndianness;

pub struct ClassicCdrDeserializer<'de> {
    bytes: &'de [u8],
    reader: &'de [u8],
    endianness: CdrEndianness,
}

impl<'de> ClassicCdrDeserializer<'de> {
    pub fn new(reader: &'de [u8], endianness: CdrEndianness) -> Self {
        Self {
            bytes: reader,
            reader,
            endianness,
        }
    }

    fn read_padding_of<T>(&mut self) -> Result<(), std::io::Error> {
        // Calculate the required padding to align with 1-byte, 2-byte, 4-byte, 8-byte
        // boundaries Instead of using the slow modulo operation '%', the faster
        // bit-masking is used
        let alignment = std::mem::size_of::<T>();
        let rem_mask = alignment - 1; // mask like 0x0, 0x1, 0x3, 0x7
        let mut padding: [u8; 8] = [0; 8];
        let pos = self.bytes.len() - self.reader.len();
        match pos & rem_mask {
            0 => Ok(()),
            n @ 1..=7 => {
                let amt = alignment - n;
                self.reader.read_exact(&mut padding[..amt])?;
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

impl<'de> CdrDeserializer<'de> for ClassicCdrDeserializer<'de> {
    fn deserialize_bool(&mut self) -> Result<bool, std::io::Error> {
        let value: u8 = CdrDeserialize::deserialize(self)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid bool encoding. Value {}", value),
            )),
        }
    }

    fn deserialize_i8(&mut self) -> Result<i8, std::io::Error> {
        self.read_padding_of::<i8>()?;
        let mut bytes = [0; 1];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(i8::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(i8::from_be_bytes(bytes)),
        }
    }

    fn deserialize_i16(&mut self) -> Result<i16, std::io::Error> {
        self.read_padding_of::<i16>()?;
        let mut bytes = [0; 2];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(i16::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(i16::from_be_bytes(bytes)),
        }
    }

    fn deserialize_i32(&mut self) -> Result<i32, std::io::Error> {
        self.read_padding_of::<i32>()?;
        let mut bytes = [0; 4];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(i32::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(i32::from_be_bytes(bytes)),
        }
    }

    fn deserialize_i64(&mut self) -> Result<i64, std::io::Error> {
        self.read_padding_of::<i64>()?;
        let mut bytes = [0; 8];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(i64::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(i64::from_be_bytes(bytes)),
        }
    }

    fn deserialize_u8(&mut self) -> Result<u8, std::io::Error> {
        self.read_padding_of::<u8>()?;
        let mut bytes = [0; 1];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(u8::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(u8::from_be_bytes(bytes)),
        }
    }

    fn deserialize_u16(&mut self) -> Result<u16, std::io::Error> {
        self.read_padding_of::<u16>()?;
        let mut bytes = [0; 2];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(u16::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(u16::from_be_bytes(bytes)),
        }
    }

    fn deserialize_u32(&mut self) -> Result<u32, std::io::Error> {
        self.read_padding_of::<u32>()?;
        let mut bytes = [0; 4];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(u32::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(u32::from_be_bytes(bytes)),
        }
    }

    fn deserialize_u64(&mut self) -> Result<u64, std::io::Error> {
        self.read_padding_of::<u64>()?;
        let mut bytes = [0; 8];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(u64::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(u64::from_be_bytes(bytes)),
        }
    }

    fn deserialize_f32(&mut self) -> Result<f32, std::io::Error> {
        self.read_padding_of::<f32>()?;
        let mut bytes = [0; 4];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(f32::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(f32::from_be_bytes(bytes)),
        }
    }

    fn deserialize_f64(&mut self) -> Result<f64, std::io::Error> {
        self.read_padding_of::<f64>()?;
        let mut bytes = [0; 8];
        self.reader.read_exact(&mut bytes)?;
        match self.endianness {
            CdrEndianness::LittleEndian => Ok(f64::from_le_bytes(bytes)),
            CdrEndianness::BigEndian => Ok(f64::from_be_bytes(bytes)),
        }
    }

    fn deserialize_char(&mut self) -> Result<char, std::io::Error> {
        let value: u8 = CdrDeserialize::deserialize(self)?;
        // CDR only accepts ascii characters.
        // The encoding length must be 1 which in UTF-8 is represented by a 0 on the MSB
        if value & 0b1000_0000 == 0 {
            Ok(value as char)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid bool encoding. Value {}", value),
            ))
        }
    }

    fn deserialize_string(&mut self) -> Result<String, std::io::Error> {
        let len: u32 = CdrDeserialize::deserialize(self)?;
        let mut buf = vec![0_u8; len as usize];
        self.reader.read_exact(&mut buf)?;
        buf.pop(); // removes a terminating null character
        String::from_utf8(buf).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("InvalidUtf8Encoding: {}", e.utf8_error()),
            )
        })
    }

    fn deserialize_seq<T>(&mut self) -> Result<Vec<T>, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let len: u32 = CdrDeserialize::deserialize(self)?;
        let mut seq = Vec::with_capacity(len as usize);
        for _ in 0..len {
            seq.push(CdrDeserialize::deserialize(self)?);
        }
        Ok(seq)
    }

    fn deserialize_array<const N: usize, T>(&mut self) -> Result<[T; N], std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut seq = Vec::with_capacity(N);
        for _ in 0..N {
            seq.push(CdrDeserialize::deserialize(self)?);
        }
        Ok(seq
            .try_into()
            .map_err(|_| ())
            .expect("Must convert due to for loop succeeding"))
    }

    fn deserialize_bytes(&mut self) -> Result<&'de [u8], std::io::Error> {
        let len: u32 = CdrDeserialize::deserialize(&mut *self)?;
        let start_pos = self.bytes.len() - self.reader.len();
        let end_pos = start_pos + len as usize;
        if self.bytes.len() >= end_pos {
            let buf = &self.bytes[start_pos..end_pos];
            self.reader.consume(len as usize);
            Ok(buf)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Byte array too small for received length".to_string(),
            ))
        }
    }

    fn deserialize_byte_array<const N: usize>(&mut self) -> Result<&'de [u8; N], std::io::Error> {
        let start_pos = self.bytes.len() - self.reader.len();
        let end_pos = start_pos + N;
        if self.bytes.len() >= end_pos {
            let buf = self.bytes[start_pos..end_pos]
                .try_into()
                .expect("Slice length is guaranteed");
            self.reader.consume(N);
            Ok(buf)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Byte array too small for received length".to_string(),
            ))
        }
    }

    fn deserialize_unit(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deserialize_be<'de, T>(bytes: &'de [u8]) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de> + ?Sized,
    {
        let mut deserializer = ClassicCdrDeserializer::new(bytes, CdrEndianness::BigEndian);
        Ok(T::deserialize(&mut deserializer)?)
    }

    fn deserialize_le<'de, T>(bytes: &'de [u8]) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de> + ?Sized,
    {
        let mut deserializer = ClassicCdrDeserializer::new(bytes, CdrEndianness::LittleEndian);
        Ok(T::deserialize(&mut deserializer)?)
    }

    #[test]
    fn deserialize_octet() {
        let v = 32u8;
        assert_eq!(deserialize_be::<u8>(&[0x20]).unwrap(), v);
        assert_eq!(deserialize_le::<u8>(&[0x20]).unwrap(), v);
    }

    #[test]
    fn deserialize_char() {
        let v = 'Z';
        assert_eq!(deserialize_be::<char>(&[0x5a]).unwrap(), v);
        assert_eq!(deserialize_le::<char>(&[0x5a]).unwrap(), v);
    }

    #[test]
    fn deserialize_wchar() {
        let v = '😞';
        let mut buf = [0; 4];
        v.encode_utf8(&mut buf);
        assert!(deserialize_be::<char>(&buf).is_err());
        assert!(deserialize_le::<char>(&buf).is_err());
    }

    #[test]
    fn deserialize_ushort() {
        let v = 65500u16;
        assert_eq!(deserialize_be::<u16>(&[0xff, 0xdc]).unwrap(), v);
        assert_eq!(deserialize_le::<u16>(&[0xdc, 0xff]).unwrap(), v);
    }

    #[test]
    fn deserialize_short() {
        let v = -32700i16;
        assert_eq!(deserialize_be::<i16>(&[0x80, 0x44]).unwrap(), v);
        assert_eq!(deserialize_le::<i16>(&[0x44, 0x80]).unwrap(), v);
    }

    #[test]
    fn deserialize_ulong() {
        let v = 4294967200u32;
        assert_eq!(deserialize_be::<u32>(&[0xff, 0xff, 0xff, 0xa0]).unwrap(), v);
        assert_eq!(deserialize_le::<u32>(&[0xa0, 0xff, 0xff, 0xff]).unwrap(), v);
    }

    #[test]
    fn deserialize_long() {
        let v = -2147483600i32;
        assert_eq!(deserialize_be::<i32>(&[0x80, 0x00, 0x00, 0x30]).unwrap(), v);
        assert_eq!(deserialize_le::<i32>(&[0x30, 0x00, 0x00, 0x80]).unwrap(), v);
    }

    #[test]
    fn deserialize_ulonglong() {
        let v = 18446744073709551600u64;
        assert_eq!(
            deserialize_be::<u64>(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<u64>(&[0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_longlong() {
        let v = -9223372036800i64;
        assert_eq!(
            deserialize_be::<i64>(&[0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<i64>(&[0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_float() {
        let v = std::f32::MIN_POSITIVE;
        assert_eq!(deserialize_be::<f32>(&[0x00, 0x80, 0x00, 0x00]).unwrap(), v);
        assert_eq!(deserialize_le::<f32>(&[0x00, 0x00, 0x80, 0x00]).unwrap(), v);
    }

    #[test]
    fn deserialize_double() {
        let v = std::f64::MIN_POSITIVE;
        assert_eq!(
            deserialize_be::<f64>(&[0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<f64>(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_bool() {
        let v = true;
        assert_eq!(deserialize_be::<bool>(&[0x01]).unwrap(), v);
        assert_eq!(deserialize_le::<bool>(&[0x01]).unwrap(), v);
        assert!(deserialize_le::<bool>(&[0x05]).is_err());
    }

    #[test]
    fn deserialize_string() {
        let v = "Hola a todos, esto es un test";
        assert_eq!(
            deserialize_be::<String>(&[
                0x00, 0x00, 0x00, 0x1e, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<String>(&[
                0x1e, 0x00, 0x00, 0x00, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_wstring() {
        let v = "みなさんこんにちは。これはテストです。";
        assert!(deserialize_be::<String>(v.as_bytes()).is_err());
        assert!(deserialize_le::<String>(v.as_bytes()).is_err());
    }

    #[test]
    fn deserialize_empty_string() {
        let v = "";
        assert_eq!(
            deserialize_be::<String>(&[0x00, 0x00, 0x00, 0x01, 0x00]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<String>(&[0x01, 0x00, 0x00, 0x00, 0x00]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_octet_array() {
        let v = [1u8, 2, 3, 4, 5];
        assert_eq!(
            deserialize_be::<[u8; 5]>(&[0x01, 0x02, 0x03, 0x04, 0x05]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<[u8; 5]>(&[0x01, 0x02, 0x03, 0x04, 0x05]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_char_array() {
        let v = ['A', 'B', 'C', 'D', 'E'];
        assert_eq!(
            deserialize_be::<[char; 5]>(&[0x41, 0x42, 0x43, 0x44, 0x45]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<[char; 5]>(&[0x41, 0x42, 0x43, 0x44, 0x45]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_string_array() {
        let v = ["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            deserialize_be::<[String; 5]>(&[
                0x00, 0x00, 0x00, 0x05, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x04, //
                0x42, 0x59, 0x45, 0x00, //
                0x00, 0x00, 0x00, 0x08, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<[String; 5]>(&[
                0x05, 0x00, 0x00, 0x00, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x04, 0x00, 0x00, 0x00, //
                0x42, 0x59, 0x45, 0x00, //
                0x08, 0x00, 0x00, 0x00, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_ushort_sequence() {
        let v = vec![65500u16, 65501, 65502, 65503, 65504];
        assert_eq!(
            deserialize_be::<Vec<u16>>(&[
                0x00, 0x00, 0x00, 0x05, //
                0xff, 0xdc, //
                0xff, 0xdd, //
                0xff, 0xde, //
                0xff, 0xdf, //
                0xff, 0xe0
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<Vec<u16>>(&[
                0x05, 0x00, 0x00, 0x00, //
                0xdc, 0xff, //
                0xdd, 0xff, //
                0xde, 0xff, //
                0xdf, 0xff, //
                0xe0, 0xff
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_string_sequence() {
        let v = vec!["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            deserialize_be::<Vec<String>>(&[
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x05, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x06, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x00, 0x00, 0x00, 0x04, //
                0x42, 0x59, 0x45, 0x00, //
                0x00, 0x00, 0x00, 0x08, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<Vec<String>>(&[
                0x05, 0x00, 0x00, 0x00, //
                0x05, 0x00, 0x00, 0x00, //
                0x48, 0x4f, 0x4c, 0x41, 0x00, //
                0x00, 0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x41, 0x44, 0x49, 0x4f, 0x53, 0x00, //
                0x00, 0x00, //
                0x06, 0x00, 0x00, 0x00, //
                0x48, 0x45, 0x4c, 0x4c, 0x4f, 0x00, //
                0x00, 0x00, //
                0x04, 0x00, 0x00, 0x00, //
                0x42, 0x59, 0x45, 0x00, //
                0x08, 0x00, 0x00, 0x00, //
                0x47, 0x4f, 0x4f, 0x44, 0x42, 0x59, 0x45, 0x00,
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_bytes() {
        let v = &[1u8, 2, 3, 4, 5];
        assert_eq!(
            deserialize_be::<&[u8]>(&[
                0x00, 0x00, 0x00, 0x05, //
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<&[u8]>(&[
                0x05, 0x00, 0x00, 0x00, //
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_byte_array() {
        let v = &[1u8, 2, 3, 4, 5];
        assert_eq!(
            deserialize_be::<&[u8; 5]>(&[
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_le::<&[u8; 5]>(&[
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
    }
}
