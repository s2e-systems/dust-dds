use std::io::Write;

use crate::xtypes::{serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer};

const PL_CDR_LE: [u8; 2] = [0x00, 0x03];
const REPRESENTATION_OPTIONS: [u8; 2] = [0x00, 0x00];
const PID_SENTINEL: i16 = 1;

pub struct ParameterListCdrSerializer {
    pub writer: Vec<u8>,
}

impl ParameterListCdrSerializer {
    pub fn new() -> Self {
        Self { writer: Vec::new() }
    }
}

impl ParameterListCdrSerializer {
    pub fn write_header(&mut self) -> Result<(), std::io::Error> {
        self.writer.write_all(&PL_CDR_LE)?;
        self.writer.write_all(&REPRESENTATION_OPTIONS)
    }
    pub fn write_sentinel(&mut self) -> Result<(), std::io::Error> {
        self.writer.write_all(&PID_SENTINEL.to_le_bytes())?;
        self.writer.write_all(&0_u16.to_le_bytes())
    }

    pub fn write<T>(&mut self, id: i16, value: &T) -> Result<(), std::io::Error>
    where
        T: XTypesSerialize,
    {
        let mut data = Vec::new();

        let mut data_serializer = Xcdr1LeSerializer::new(&mut data);
        value.serialize(&mut data_serializer).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("XTypes error: {:?}", e),
            )
        })?;

        let length_without_padding = data.len();
        let padding_length = (4 - length_without_padding % 4) & 3;
        let length = length_without_padding + padding_length;

        if length > u16::MAX as usize {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Serialized parameter ID {} with serialized size {} exceeds maximum parameter size of {}", id, length, u16::MAX)));
        }

        self.writer.write_all(&id.to_le_bytes())?;
        self.writer.write_all(&(length as u16).to_le_bytes())?;

        self.writer.write_all(&data)?;

        match padding_length {
            1 => self.writer.write_all(&[0u8; 1])?,
            2 => self.writer.write_all(&[0u8; 2])?,
            3 => self.writer.write_all(&[0u8; 3])?,
            _ => self.writer.write_all(&[0u8; 0])?,
        }

        Ok(())
    }

    pub fn write_with_default<T>(
        &mut self,
        id: i16,
        value: &T,
        default: &T,
    ) -> Result<(), std::io::Error>
    where
        T: XTypesSerialize + PartialEq,
    {
        if value != default {
            self.write(id, value)?;
        }
        Ok(())
    }

    pub fn write_collection<T>(&mut self, id: i16, value_list: &[T]) -> Result<(), std::io::Error>
    where
        T: XTypesSerialize,
    {
        for value in value_list {
            self.write(id, value)?;
        }
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::serialized_payload::parameter_list::serialize::ParameterListSerialize;

//     use super::*;

    // fn serialize_le<T>(v: &T) -> Result<Vec<u8>, std::io::Error>
    // where
    //     T: ParameterListSerialize,
    // {
    //     let mut writer = Vec::new();
    //     let mut serializer = ParameterListCdrSerializer::new(&mut writer);
    //     v.serialize(&mut serializer)?;
    //     Ok(writer)
    // }

    // #[test]
    // fn write_parameter_list_without_defaults() {
    //     struct ParameterListWithoutDefaults {
    //         a: i32,
    //         b: String,
    //         c: [u16; 4],
    //     }

    //     impl ParameterListSerialize for ParameterListWithoutDefaults {
    //         fn serialize(
    //             &self,
    //             serializer: &mut impl ParameterListSerializer,
    //         ) -> Result<(), std::io::Error> {
    //             serializer.write(1, &self.a)?;
    //             serializer.write(2, &self.b)?;
    //             serializer.write(3, &self.c)?;
    //             Ok(())
    //         }
    //     }

    //     let value = ParameterListWithoutDefaults {
    //         a: 100,
    //         b: "Hello".to_string(),
    //         c: [1, 2, 3, 4],
    //     };

    //     assert_eq!(
    //         serialize_be(&value).unwrap(),
    //         vec![
    //             0, 1, 0, 4, // PID, length
    //             0, 0, 0, 100, // u32
    //             0, 2, 0, 12, // PID, length
    //             0, 0, 0, 6, // String length
    //             b'H', b'e', b'l', b'l', //
    //             b'o', 0, 0, 0, // 2 bytes padding
    //             0, 3, 0, 8, // PID, length
    //             0, 1, 0, 2, //
    //             0, 3, 0, 4, //
    //         ]
    //     );
    //     assert_eq!(
    //         serialize_le(&value).unwrap(),
    //         vec![
    //             1, 0, 4, 0, // PID, length
    //             100, 0, 0, 0, // u32
    //             2, 0, 12, 0, // PID, length
    //             6, 0, 0, 0, // String length
    //             b'H', b'e', b'l', b'l', //
    //             b'o', 0, 0, 0, // 2 bytes padding
    //             3, 0, 8, 0, // PID, length
    //             1, 0, 2, 0, //
    //             3, 0, 4, 0, //
    //         ]
    //     );
    // }
// }
