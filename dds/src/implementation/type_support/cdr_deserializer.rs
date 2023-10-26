use std::{
    convert::TryInto,
    io::{BufRead, Read},
    marker::PhantomData,
};

use byteorder::{ByteOrder, ReadBytesExt};

use crate::{
    infrastructure::error::{DdsError, DdsResult},
    topic_definition::cdr_type::{CdrDeserialize, CdrDeserializer},
};

pub struct CdrDataDeserializer<'de, E> {
    bytes: &'de [u8],
    reader: &'de [u8],
    phantom: PhantomData<E>,
}

impl<'de, E> CdrDataDeserializer<'de, E>
where
    E: ByteOrder,
{
    pub fn new(reader: &'de [u8]) -> Self {
        Self {
            bytes: reader,
            reader,
            phantom: PhantomData,
        }
    }

    fn read_padding_of<T>(&mut self) -> DdsResult<()> {
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

impl<'de, E> CdrDeserializer<'de> for CdrDataDeserializer<'de, E>
where
    E: ByteOrder,
{
    fn deserialize_bool(&mut self) -> DdsResult<bool> {
        let value: u8 = CdrDeserialize::deserialize(self)?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DdsError::Error(format!(
                "Invalid bool encoding. Value {}",
                value
            ))),
        }
    }

    fn deserialize_i8(&mut self) -> DdsResult<i8> {
        self.read_padding_of::<i8>()?;
        Ok(self.reader.read_i8()?)
    }

    fn deserialize_i16(&mut self) -> DdsResult<i16> {
        self.read_padding_of::<i16>()?;
        Ok(self.reader.read_i16::<E>()?)
    }

    fn deserialize_i32(&mut self) -> DdsResult<i32> {
        self.read_padding_of::<i32>()?;
        Ok(self.reader.read_i32::<E>()?)
    }

    fn deserialize_i64(&mut self) -> DdsResult<i64> {
        self.read_padding_of::<i64>()?;
        Ok(self.reader.read_i64::<E>()?)
    }

    fn deserialize_u8(&mut self) -> DdsResult<u8> {
        self.read_padding_of::<u8>()?;
        Ok(self.reader.read_u8()?)
    }

    fn deserialize_u16(&mut self) -> DdsResult<u16> {
        self.read_padding_of::<u16>()?;
        Ok(self.reader.read_u16::<E>()?)
    }

    fn deserialize_u32(&mut self) -> DdsResult<u32> {
        self.read_padding_of::<u32>()?;
        Ok(self.reader.read_u32::<E>()?)
    }

    fn deserialize_u64(&mut self) -> DdsResult<u64> {
        self.read_padding_of::<u64>()?;
        Ok(self.reader.read_u64::<E>()?)
    }

    fn deserialize_f32(&mut self) -> DdsResult<f32> {
        self.read_padding_of::<f32>()?;
        Ok(self.reader.read_f32::<E>()?)
    }

    fn deserialize_f64(&mut self) -> DdsResult<f64> {
        self.read_padding_of::<f64>()?;
        Ok(self.reader.read_f64::<E>()?)
    }

    fn deserialize_char(&mut self) -> DdsResult<char> {
        let value: u8 = CdrDeserialize::deserialize(self)?;
        // CDR only accepts ascii characters.
        // The encoding length must be 1 which in UTF-8 is represented by a 0 on the MSB
        if value & 0b1000_0000 == 0 {
            Ok(value as char)
        } else {
            Err(DdsError::Error(format!(
                "Invalid bool encoding. Value {}",
                value
            )))
        }
    }

    fn deserialize_string(&mut self) -> DdsResult<String> {
        let len: u32 = CdrDeserialize::deserialize(self)?;
        let mut buf = vec![0_u8; len as usize];
        self.reader.read_exact(&mut buf)?;
        buf.pop(); // removes a terminating null character
        String::from_utf8(buf)
            .map_err(|e| DdsError::Error(format!("InvalidUtf8Encoding: {}", e.utf8_error())))
    }

    fn deserialize_seq<T>(&mut self) -> DdsResult<Vec<T>>
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

    fn deserialize_array<const N: usize, T>(&mut self) -> DdsResult<[T; N]>
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

    fn deserialize_bytes(&mut self) -> DdsResult<&'de [u8]> {
        let len: u32 = CdrDeserialize::deserialize(&mut *self)?;
        let start_pos = self.bytes.len() - self.reader.len();
        let end_pos = start_pos + len as usize;
        if self.bytes.len() >= end_pos {
            let buf = &self.bytes[start_pos..end_pos];
            self.reader.consume(len as usize);
            Ok(buf)
        } else {
            Err(DdsError::Error(format!(
                "Byte array too small for received length"
            )))
        }
    }

    fn deserialize_byte_array<const N: usize>(&mut self) -> DdsResult<&'de [u8; N]> {
        let start_pos = self.bytes.len() - self.reader.len();
        let end_pos = start_pos + N;
        if self.bytes.len() >= end_pos {
            let buf = self.bytes[start_pos..end_pos]
                .try_into()
                .expect("Slice length is guaranteed");
            self.reader.consume(N);
            Ok(buf)
        } else {
            Err(DdsError::Error(format!(
                "Byte array too small for received length"
            )))
        }
    }

    fn deserialize_unit(&mut self) -> DdsResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, LittleEndian};

    use super::*;

    fn deserialize_data<'de, T, E>(bytes: &'de [u8]) -> DdsResult<T>
    where
        E: ByteOrder,
        T: CdrDeserialize<'de> + ?Sized,
    {
        let mut deserializer = CdrDataDeserializer::<E>::new(bytes);
        Ok(T::deserialize(&mut deserializer)?)
    }

    #[test]
    fn deserialize_octet() {
        let v = 32u8;
        assert_eq!(deserialize_data::<u8, BigEndian>(&[0x20]).unwrap(), v);
        assert_eq!(deserialize_data::<u8, LittleEndian>(&[0x20]).unwrap(), v);
    }

    #[test]
    fn deserialize_char() {
        let v = 'Z';
        assert_eq!(deserialize_data::<char, BigEndian>(&[0x5a]).unwrap(), v);
        assert_eq!(deserialize_data::<char, LittleEndian>(&[0x5a]).unwrap(), v);
    }

    #[test]
    fn deserialize_wchar() {
        let v = '😞';
        let mut buf = [0; 4];
        v.encode_utf8(&mut buf);
        assert!(deserialize_data::<char, BigEndian>(&buf).is_err());
        assert!(deserialize_data::<char, LittleEndian>(&buf).is_err());
    }

    #[test]
    fn deserialize_ushort() {
        let v = 65500u16;
        assert_eq!(
            deserialize_data::<u16, BigEndian>(&[0xff, 0xdc]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<u16, LittleEndian>(&[0xdc, 0xff]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_short() {
        let v = -32700i16;
        assert_eq!(
            deserialize_data::<i16, BigEndian>(&[0x80, 0x44]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<i16, LittleEndian>(&[0x44, 0x80]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_ulong() {
        let v = 4294967200u32;
        assert_eq!(
            deserialize_data::<u32, BigEndian>(&[0xff, 0xff, 0xff, 0xa0]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<u32, LittleEndian>(&[0xa0, 0xff, 0xff, 0xff]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_long() {
        let v = -2147483600i32;
        assert_eq!(
            deserialize_data::<i32, BigEndian>(&[0x80, 0x00, 0x00, 0x30]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<i32, LittleEndian>(&[0x30, 0x00, 0x00, 0x80]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_ulonglong() {
        let v = 18446744073709551600u64;
        assert_eq!(
            deserialize_data::<u64, BigEndian>(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0])
                .unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<u64, LittleEndian>(&[
                0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_longlong() {
        let v = -9223372036800i64;
        assert_eq!(
            deserialize_data::<i64, BigEndian>(&[0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40])
                .unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<i64, LittleEndian>(&[
                0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_float() {
        let v = std::f32::MIN_POSITIVE;
        assert_eq!(
            deserialize_data::<f32, BigEndian>(&[0x00, 0x80, 0x00, 0x00]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<f32, LittleEndian>(&[0x00, 0x00, 0x80, 0x00]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_double() {
        let v = std::f64::MIN_POSITIVE;
        assert_eq!(
            deserialize_data::<f64, BigEndian>(&[0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<f64, LittleEndian>(&[
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00
            ])
            .unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_bool() {
        let v = true;
        assert_eq!(deserialize_data::<bool, BigEndian>(&[0x01]).unwrap(), v);
        assert_eq!(deserialize_data::<bool, LittleEndian>(&[0x01]).unwrap(), v);
        assert!(deserialize_data::<bool, LittleEndian>(&[0x05]).is_err());
    }

    #[test]
    fn deserialize_string() {
        let v = "Hola a todos, esto es un test";
        assert_eq!(
            deserialize_data::<String, BigEndian>(&[
                0x00, 0x00, 0x00, 0x1e, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<String, LittleEndian>(&[
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
        assert!(deserialize_data::<String, BigEndian>(v.as_bytes()).is_err());
        assert!(deserialize_data::<String, LittleEndian>(v.as_bytes()).is_err());
    }

    #[test]
    fn deserialize_empty_string() {
        let v = "";
        assert_eq!(
            deserialize_data::<String, BigEndian>(&[0x00, 0x00, 0x00, 0x01, 0x00]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<String, LittleEndian>(&[0x01, 0x00, 0x00, 0x00, 0x00]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_octet_array() {
        let v = [1u8, 2, 3, 4, 5];
        assert_eq!(
            deserialize_data::<[u8; 5], BigEndian>(&[0x01, 0x02, 0x03, 0x04, 0x05]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<[u8; 5], LittleEndian>(&[0x01, 0x02, 0x03, 0x04, 0x05]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_char_array() {
        let v = ['A', 'B', 'C', 'D', 'E'];
        assert_eq!(
            deserialize_data::<[char; 5], BigEndian>(&[0x41, 0x42, 0x43, 0x44, 0x45]).unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<[char; 5], LittleEndian>(&[0x41, 0x42, 0x43, 0x44, 0x45]).unwrap(),
            v
        );
    }

    #[test]
    fn deserialize_string_array() {
        let v = ["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            deserialize_data::<[String; 5], BigEndian>(&[
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
            deserialize_data::<[String; 5], LittleEndian>(&[
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
            deserialize_data::<Vec<u16>, BigEndian>(&[
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
            deserialize_data::<Vec<u16>, LittleEndian>(&[
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
            deserialize_data::<Vec<String>, BigEndian>(&[
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
            deserialize_data::<Vec<String>, LittleEndian>(&[
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
            deserialize_data::<&[u8], BigEndian>(&[
                0x00, 0x00, 0x00, 0x05, //
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<&[u8], LittleEndian>(&[
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
            deserialize_data::<&[u8; 5], BigEndian>(&[
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
        assert_eq!(
            deserialize_data::<&[u8; 5], LittleEndian>(&[
                0x01, 0x02, 0x03, 0x04, 0x05 //
            ])
            .unwrap(),
            v
        );
    }
}
