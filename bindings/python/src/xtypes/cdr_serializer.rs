use super::endianness::CdrEndianness;
use dust_dds::serialized_payload::cdr::{serialize::CdrSerialize, serializer::CdrSerializer};

pub struct ClassicCdrSerializer<W> {
    writer: W,
    pos: usize,
    endianness: CdrEndianness,
}

impl<W> ClassicCdrSerializer<W>
where
    W: std::io::Write,
{
    pub fn new(writer: W, endianness: CdrEndianness) -> Self {
        Self {
            writer,
            pos: 0,
            endianness,
        }
    }

    fn add_pos(&mut self, size: usize) {
        self.pos += size;
    }

    fn set_pos_of<T>(&mut self) -> Result<(), std::io::Error> {
        self.write_padding_of::<T>()?;
        self.add_pos(std::mem::size_of::<T>());
        Ok(())
    }

    fn write_padding_of<T>(&mut self) -> Result<(), std::io::Error> {
        // Calculate the required padding to align with 1-byte, 2-byte, 4-byte, 8-byte boundaries.
        // Instead of using the slow modulo operation '%', the faster bit-masking is used
        const PADDING: [u8; 8] = [0; 8];
        let alignment = std::mem::size_of::<T>();
        let rem_mask = alignment - 1; // mask like 0x0, 0x1, 0x3, 0x7
        match (self.pos) & rem_mask {
            0 => Ok(()),
            n @ 1..=7 => {
                let amt = alignment - n;
                self.pos += amt;
                self.writer.write_all(&PADDING[..amt]).map_err(Into::into)
            }
            _ => unreachable!(),
        }
    }
}

impl<W> CdrSerializer for ClassicCdrSerializer<W>
where
    W: std::io::Write,
{
    fn serialize_bool(&mut self, v: bool) -> Result<(), std::io::Error> {
        self.serialize_u8(v as u8)
    }

    fn serialize_i8(&mut self, v: i8) -> Result<(), std::io::Error> {
        self.set_pos_of::<i8>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_i16(&mut self, v: i16) -> Result<(), std::io::Error> {
        self.set_pos_of::<i16>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_i32(&mut self, v: i32) -> Result<(), std::io::Error> {
        self.set_pos_of::<i32>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), std::io::Error> {
        self.set_pos_of::<i64>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_u8(&mut self, v: u8) -> Result<(), std::io::Error> {
        self.set_pos_of::<u8>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_u16(&mut self, v: u16) -> Result<(), std::io::Error> {
        self.set_pos_of::<u16>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_u32(&mut self, v: u32) -> Result<(), std::io::Error> {
        self.set_pos_of::<u32>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), std::io::Error> {
        self.set_pos_of::<u64>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_f32(&mut self, v: f32) -> Result<(), std::io::Error> {
        self.set_pos_of::<f32>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_f64(&mut self, v: f64) -> Result<(), std::io::Error> {
        self.set_pos_of::<f64>()?;
        match self.endianness {
            CdrEndianness::LittleEndian => self.writer.write_all(&v.to_le_bytes()),
            CdrEndianness::BigEndian => self.writer.write_all(&v.to_be_bytes()),
        }
    }

    fn serialize_char(&mut self, v: char) -> Result<(), std::io::Error> {
        if !v.is_ascii() {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid character: {}", v),
            ))
        } else {
            let mut buf = [0u8; 1];
            v.encode_utf8(&mut buf);
            self.add_pos(1);
            self.writer.write_all(&buf)?;
            Ok(())
        }
    }

    fn serialize_str(&mut self, v: &str) -> Result<(), std::io::Error> {
        if !v.is_ascii() {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid string: {}", v),
            ))
        } else {
            let terminating_char = [0u8];
            let l = v.len() + terminating_char.len();
            if l > u32::MAX as usize {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("String too long. String size {}, maximum {}", l, u32::MAX,),
                ))
            } else {
                self.serialize_u32(l as u32)?;
                self.add_pos(l);
                self.writer.write_all(v.as_bytes())?;
                self.writer.write_all(&terminating_char)?;
                Ok(())
            }
        }
    }

    fn serialize_seq(&mut self, v: &[impl CdrSerialize]) -> Result<(), std::io::Error> {
        let l = v.len();
        if l > u32::MAX as usize {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("String too long. String size {}, maximum {}", l, u32::MAX,),
            ))
        } else {
            self.serialize_u32(l as u32)?;
            for e in v {
                e.serialize(self)?;
            }
            Ok(())
        }
    }

    fn serialize_array<const N: usize>(
        &mut self,
        v: &[impl CdrSerialize; N],
    ) -> Result<(), std::io::Error> {
        for e in v {
            e.serialize(self)?;
        }
        Ok(())
    }

    fn serialize_unit(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }

    fn serialize_bytes(&mut self, v: &[u8]) -> Result<(), std::io::Error> {
        self.writer.write_all(v)
    }

    fn serialize_byte_array<const N: usize>(&mut self, v: &[u8; N]) -> Result<(), std::io::Error> {
        self.writer.write_all(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn serialize_be<T>(v: &T) -> Result<Vec<u8>, std::io::Error>
    where
        T: CdrSerialize + ?Sized,
    {
        let mut writer = Vec::new();
        let mut serializer = ClassicCdrSerializer::new(&mut writer, CdrEndianness::BigEndian);
        v.serialize(&mut serializer)?;
        Ok(writer)
    }

    fn serialize_le<T>(v: &T) -> Result<Vec<u8>, std::io::Error>
    where
        T: CdrSerialize + ?Sized,
    {
        let mut writer = Vec::new();
        let mut serializer = ClassicCdrSerializer::new(&mut writer, CdrEndianness::LittleEndian);
        v.serialize(&mut serializer)?;
        Ok(writer)
    }

    #[test]
    fn serialize_octet() {
        let v = 32u8;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0x20]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0x20]);
    }

    #[test]
    fn serialize_char() {
        let v = 'Z';
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0x5a]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0x5a]);
    }

    #[test]
    fn serialize_wchar() {
        let v = 'Å';
        assert!(serialize_be::<_>(&v).is_err());
        assert!(serialize_le::<_>(&v).is_err());
    }

    #[test]
    fn serialize_ushort() {
        let v = 65500u16;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0xff, 0xdc]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0xdc, 0xff]);
    }

    #[test]
    fn serialize_short() {
        let v = -32700i16;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0x80, 0x44]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0x44, 0x80]);
    }

    #[test]
    fn serialize_ulong() {
        let v = 4294967200u32;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0xff, 0xff, 0xff, 0xa0]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0xa0, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn serialize_long() {
        let v = -2147483600i32;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0x80, 0x00, 0x00, 0x30]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0x30, 0x00, 0x00, 0x80]);
    }

    #[test]
    fn serialize_ulonglong() {
        let v = 18446744073709551600u64;
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_longlong() {
        let v = -9223372036800i64;
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff]
        );
    }

    #[test]
    fn serialize_float() {
        let v = std::f32::MIN_POSITIVE;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0x00, 0x80, 0x00, 0x00]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0x00, 0x00, 0x80, 0x00]);
    }

    #[test]
    fn serialize_double() {
        let v = std::f64::MIN_POSITIVE;
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00]
        );
    }

    #[test]
    fn serialize_bool() {
        let v = true;
        assert_eq!(serialize_be::<_>(&v).unwrap(), vec![0x01]);
        assert_eq!(serialize_le::<_>(&v).unwrap(), vec![0x01]);
    }

    #[test]
    fn serialize_string() {
        let v = "Hola a todos, esto es un test";
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x1e, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x1e, 0x00, 0x00, 0x00, 0x48, 0x6f, 0x6c, 0x61, 0x20, 0x61, 0x20, 0x74, 0x6f, 0x64,
                0x6f, 0x73, 0x2c, 0x20, 0x65, 0x73, 0x74, 0x6f, 0x20, 0x65, 0x73, 0x20, 0x75, 0x6e,
                0x20, 0x74, 0x65, 0x73, 0x74, 0x00,
            ]
        );
    }

    #[test]
    fn serialize_wstring() {
        let v = "みなさんこんにちは。これはテストです。";
        assert!(serialize_be::<_>(&v).is_err());
        assert!(serialize_le::<_>(&v).is_err());
    }

    #[test]
    fn serialize_empty_string() {
        let v = "";
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0x00, 0x00, 0x00, 0x01, 0x00]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0x01, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn serialize_octet_array() {
        let v = [1u8, 2, 3, 4, 5];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0x01, 0x02, 0x03, 0x04, 0x05]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0x01, 0x02, 0x03, 0x04, 0x05]
        );
    }

    #[test]
    fn serialize_char_array() {
        let v = ['A', 'B', 'C', 'D', 'E'];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0x41, 0x42, 0x43, 0x44, 0x45]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0x41, 0x42, 0x43, 0x44, 0x45]
        );
    }

    #[test]
    fn serialize_ushort_array() {
        let v = [65500u16, 65501, 65502, 65503, 65504];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0xff, 0xdc, //
                0xff, 0xdd, //
                0xff, 0xde, //
                0xff, 0xdf, //
                0xff, 0xe0
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0xdc, 0xff, //
                0xdd, 0xff, //
                0xde, 0xff, //
                0xdf, 0xff, //
                0xe0, 0xff
            ]
        );
    }

    #[test]
    fn serialize_short_array() {
        let v = [-32700i16, -32701, -32702, -32703, -32704];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x80, 0x44, //
                0x80, 0x43, //
                0x80, 0x42, //
                0x80, 0x41, //
                0x80, 0x40
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x44, 0x80, //
                0x43, 0x80, //
                0x42, 0x80, //
                0x41, 0x80, //
                0x40, 0x80
            ]
        );
    }

    #[test]
    fn serialize_ulong_array() {
        let v = [
            4294967200u32,
            4294967201,
            4294967202,
            4294967203,
            4294967204,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0xff, 0xff, 0xff, 0xa0, //
                0xff, 0xff, 0xff, 0xa1, //
                0xff, 0xff, 0xff, 0xa2, //
                0xff, 0xff, 0xff, 0xa3, //
                0xff, 0xff, 0xff, 0xa4,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0xa0, 0xff, 0xff, 0xff, //
                0xa1, 0xff, 0xff, 0xff, //
                0xa2, 0xff, 0xff, 0xff, //
                0xa3, 0xff, 0xff, 0xff, //
                0xa4, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_long_array() {
        let v = [
            -2147483600,
            -2147483601,
            -2147483602,
            -2147483603,
            -2147483604,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x80, 0x00, 0x00, 0x30, //
                0x80, 0x00, 0x00, 0x2f, //
                0x80, 0x00, 0x00, 0x2e, //
                0x80, 0x00, 0x00, 0x2d, //
                0x80, 0x00, 0x00, 0x2c,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x30, 0x00, 0x00, 0x80, //
                0x2f, 0x00, 0x00, 0x80, //
                0x2e, 0x00, 0x00, 0x80, //
                0x2d, 0x00, 0x00, 0x80, //
                0x2c, 0x00, 0x00, 0x80,
            ]
        );
    }

    #[test]
    fn serialize_ulonglong_array() {
        let v = [
            18446744073709551600u64,
            18446744073709551601,
            18446744073709551602,
            18446744073709551603,
            18446744073709551604,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf1, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf2, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf4,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf4, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_longlong_array() {
        let v = [
            -9223372036800i64,
            -9223372036801,
            -9223372036802,
            -9223372036803,
            -9223372036804,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3f, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3e, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3d, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3c,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3f, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3e, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3d, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3c, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_float_array() {
        let f = std::f32::MIN_POSITIVE;

        let v = [f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x80, 0x00, 0x00, //
                0x3f, 0x80, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, //
                0x40, 0x40, 0x00, 0x00, //
                0x40, 0x80, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x80, 0x00, //
                0x00, 0x00, 0x80, 0x3f, //
                0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x40, 0x40, //
                0x00, 0x00, 0x80, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_double_array() {
        let f = std::f64::MIN_POSITIVE;

        let v = [f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_bool_array() {
        let v = [true, false, true, false, true];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![0x01, 0x00, 0x01, 0x00, 0x01]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![0x01, 0x00, 0x01, 0x00, 0x01]
        );
    }

    #[test]
    fn serialize_string_array() {
        let v = ["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
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
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
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
            ]
        );
    }

    #[test]
    fn serialize_octet_sequence() {
        let v = vec![1u8, 2, 3, 4, 5];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x01, 0x02, 0x03, 0x04, 0x05
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x01, 0x02, 0x03, 0x04, 0x05
            ]
        );
    }

    #[test]
    fn serialize_char_sequence() {
        let v = vec!['A', 'B', 'C', 'D', 'E'];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x41, 0x42, 0x43, 0x44, 0x45
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x41, 0x42, 0x43, 0x44, 0x45
            ]
        );
    }

    #[test]
    fn serialize_ushort_sequence() {
        let v = vec![65500u16, 65501, 65502, 65503, 65504];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0xff, 0xdc, //
                0xff, 0xdd, //
                0xff, 0xde, //
                0xff, 0xdf, //
                0xff, 0xe0
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0xdc, 0xff, //
                0xdd, 0xff, //
                0xde, 0xff, //
                0xdf, 0xff, //
                0xe0, 0xff
            ]
        );
    }

    #[test]
    fn serialize_short_sequence() {
        let v = vec![-32700i16, -32701, -32702, -32703, -32704];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x80, 0x44, //
                0x80, 0x43, //
                0x80, 0x42, //
                0x80, 0x41, //
                0x80, 0x40
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x44, 0x80, //
                0x43, 0x80, //
                0x42, 0x80, //
                0x41, 0x80, //
                0x40, 0x80
            ]
        );
    }

    #[test]
    fn serialize_ulong_sequence() {
        let v = vec![
            4294967200u32,
            4294967201,
            4294967202,
            4294967203,
            4294967204,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0xff, 0xff, 0xff, 0xa0, //
                0xff, 0xff, 0xff, 0xa1, //
                0xff, 0xff, 0xff, 0xa2, //
                0xff, 0xff, 0xff, 0xa3, //
                0xff, 0xff, 0xff, 0xa4,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0xa0, 0xff, 0xff, 0xff, //
                0xa1, 0xff, 0xff, 0xff, //
                0xa2, 0xff, 0xff, 0xff, //
                0xa3, 0xff, 0xff, 0xff, //
                0xa4, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_long_sequence() {
        let v = vec![
            -2147483600,
            -2147483601,
            -2147483602,
            -2147483603,
            -2147483604,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x80, 0x00, 0x00, 0x30, //
                0x80, 0x00, 0x00, 0x2f, //
                0x80, 0x00, 0x00, 0x2e, //
                0x80, 0x00, 0x00, 0x2d, //
                0x80, 0x00, 0x00, 0x2c,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x30, 0x00, 0x00, 0x80, //
                0x2f, 0x00, 0x00, 0x80, //
                0x2e, 0x00, 0x00, 0x80, //
                0x2d, 0x00, 0x00, 0x80, //
                0x2c, 0x00, 0x00, 0x80,
            ]
        );
    }

    #[test]
    fn serialize_ulonglong_sequence() {
        let v = vec![
            18446744073709551600u64,
            18446744073709551601,
            18446744073709551602,
            18446744073709551603,
            18446744073709551604,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x00, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf0, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf1, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf2, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf3, //
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xf4,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0xf0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf3, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, //
                0xf4, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_longlong_sequence() {
        let v = vec![
            -9223372036800i64,
            -9223372036801,
            -9223372036802,
            -9223372036803,
            -9223372036804,
        ];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x00, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x40, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3f, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3e, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3d, //
                0xff, 0xff, 0xf7, 0x9c, 0x84, 0x2f, 0xa5, 0x3c,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x40, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3f, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3e, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3d, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff, //
                0x3c, 0xa5, 0x2f, 0x84, 0x9c, 0xf7, 0xff, 0xff,
            ]
        );
    }

    #[test]
    fn serialize_float_sequence() {
        let f = std::f32::MIN_POSITIVE;

        let v = vec![f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x80, 0x00, 0x00, //
                0x3f, 0x80, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, //
                0x40, 0x40, 0x00, 0x00, //
                0x40, 0x80, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x80, 0x00, //
                0x00, 0x00, 0x80, 0x3f, //
                0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x40, 0x40, //
                0x00, 0x00, 0x80, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_double_sequence() {
        let f = std::f64::MIN_POSITIVE;

        let v = vec![f, f + 1., f + 2., f + 3., f + 4.];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
                0x40, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, //
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x40,
            ]
        );
    }

    #[test]
    fn serialize_bool_sequence() {
        let v = vec![true, false, true, false, true];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
                0x00, 0x00, 0x00, 0x05, //
                0x01, 0x00, 0x01, 0x00, 0x01
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
                0x05, 0x00, 0x00, 0x00, //
                0x01, 0x00, 0x01, 0x00, 0x01
            ]
        );
    }

    #[test]
    fn serialize_string_sequence() {
        let v = vec!["HOLA", "ADIOS", "HELLO", "BYE", "GOODBYE"];
        assert_eq!(
            serialize_be::<_>(&v).unwrap(),
            vec![
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
            ]
        );
        assert_eq!(
            serialize_le::<_>(&v).unwrap(),
            vec![
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
            ]
        );
    }
}
