use crate::cdr::{endianness::CdrEndianness, serialize::CdrSerialize, serializer::CdrSerializer};

pub struct ParameterListSerializer<'s> {
    writer: &'s mut Vec<u8>,
    endianness: CdrEndianness,
}

impl<'s> ParameterListSerializer<'s> {
    pub fn new(writer: &'s mut Vec<u8>, endianness: CdrEndianness) -> Self {
        Self { writer, endianness }
    }
}

impl ParameterListSerializer<'_> {
    pub fn write<T>(&mut self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: CdrSerialize,
    {
        let mut data = Vec::new();
        let mut data_serializer = CdrSerializer::new(&mut data, self.endianness);
        value.serialize(&mut data_serializer)?;

        let length_without_padding = data.len();
        let padding_length = (4 - length_without_padding % 4) & 3;
        let length = length_without_padding + padding_length;

        if length > u16::MAX as usize {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Serialized parameter ID {} with serialized size {} exceeds maximum parameter size of {}", id, length, u16::MAX)))
        } else {
            let mut serializer = CdrSerializer::new(self.writer, self.endianness);
            serializer.serialize_i16(id)?;
            serializer.serialize_u16(length as u16)?;

            self.writer.append(&mut data);

            match padding_length {
                1 => self.writer.extend_from_slice(&[0u8; 1]),
                2 => self.writer.extend_from_slice(&[0u8; 2]),
                3 => self.writer.extend_from_slice(&[0u8; 3]),
                _ => self.writer.extend_from_slice(&[0u8; 0]),
            }
            Ok(())
        }
    }

    pub fn write_with_default<T>(
        &mut self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), std::io::Error>
    where
        T: CdrSerialize + PartialEq,
    {
        if value != default {
            self.write(id, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parameter_list::serialize::ParameterListSerialize;

    use super::*;

    fn serialize_le<T>(v: &T) -> Result<Vec<u8>, std::io::Error>
    where
        T: ParameterListSerialize,
    {
        let mut writer = Vec::new();
        let mut serializer = ParameterListSerializer::new(&mut writer, CdrEndianness::LittleEndian);
        v.serialize(&mut serializer)?;
        Ok(writer)
    }

    fn serialize_be<T>(v: &T) -> Result<Vec<u8>, std::io::Error>
    where
        T: ParameterListSerialize,
    {
        let mut writer = Vec::new();
        let mut serializer = ParameterListSerializer::new(&mut writer, CdrEndianness::BigEndian);
        v.serialize(&mut serializer)?;
        Ok(writer)
    }

    #[test]
    fn write_parameter_list_without_defaults() {
        struct ParameterListWithoutDefaults {
            a: i32,
            b: String,
            c: [u16; 4],
        }

        impl ParameterListSerialize for ParameterListWithoutDefaults {
            fn serialize(
                &self,
                serializer: &mut ParameterListSerializer,
            ) -> Result<(), std::io::Error> {
                serializer.write(1, &self.a)?;
                serializer.write(2, &self.b)?;
                serializer.write(3, &self.c)?;
                Ok(())
            }
        }

        let value = ParameterListWithoutDefaults {
            a: 100,
            b: "Hello".to_string(),
            c: [1, 2, 3, 4],
        };

        assert_eq!(
            serialize_be(&value).unwrap(),
            vec![
                0, 1, 0, 4, // PID, length
                0, 0, 0, 100, // u32
                0, 2, 0, 12, // PID, length
                0, 0, 0, 6, // String length
                b'H', b'e', b'l', b'l', //
                b'o', 0, 0, 0, // 2 bytes padding
                0, 3, 0, 8, // PID, length
                0, 1, 0, 2, //
                0, 3, 0, 4, //
            ]
        );
        assert_eq!(
            serialize_le(&value).unwrap(),
            vec![
                1, 0, 4, 0, // PID, length
                100, 0, 0, 0, // u32
                2, 0, 12, 0, // PID, length
                6, 0, 0, 0, // String length
                b'H', b'e', b'l', b'l', //
                b'o', 0, 0, 0, // 2 bytes padding
                3, 0, 8, 0, // PID, length
                1, 0, 2, 0, //
                3, 0, 4, 0, //
            ]
        );
    }
}
