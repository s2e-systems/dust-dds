use std::io::{BufRead, Read};

use crate::{
    cdr::{deserialize::CdrDeserialize, deserializer::CdrDeserializer, endianness::CdrEndianness},
    implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
};

struct Parameter<'de> {
    id: i16,
    data: &'de [u8],
}

impl<'de> Parameter<'de> {
    fn id(&self) -> i16 {
        self.id
    }

    fn data(&self) -> &'de [u8] {
        self.data
    }
}

struct ParameterIterator<'a, 'de> {
    reader: &'a mut &'de [u8],
    endianness: CdrEndianness,
}

impl<'a, 'de> ParameterIterator<'a, 'de> {
    fn new(reader: &'a mut &'de [u8], endianness: CdrEndianness) -> Self {
        Self { reader, endianness }
    }

    fn next(&mut self) -> Result<Option<Parameter<'de>>, std::io::Error> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        let id = match self.endianness {
            CdrEndianness::LittleEndian => i16::from_le_bytes(buf),
            CdrEndianness::BigEndian => i16::from_be_bytes(buf),
        };

        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        let length = match self.endianness {
            CdrEndianness::LittleEndian => u16::from_le_bytes(buf),
            CdrEndianness::BigEndian => u16::from_be_bytes(buf),
        } as usize;

        if self.reader.len() < length {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!("Not enough data to get parameter length"),
            ))
        } else {
            if id == PID_SENTINEL {
                Ok(None)
            } else {
                let data = &self.reader[..length];
                self.reader.consume(length);
                Ok(Some(Parameter { id, data }))
            }
        }
    }
}

pub struct ParameterListDeserializer<'de> {
    bytes: &'de [u8],
    endianness: CdrEndianness,
}

impl<'de> ParameterListDeserializer<'de> {
    pub fn new(bytes: &'de [u8], endianness: CdrEndianness) -> Self {
        Self { bytes, endianness }
    }

    pub fn read<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id() == id {
                return T::deserialize(&mut CdrDeserializer::new(p.data(), self.endianness));
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Parameter with id {} not found", id),
        ))
    }

    pub fn read_with_default<T>(&self, id: i16, default: T) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id() == id {
                return T::deserialize(&mut CdrDeserializer::new(p.data(), self.endianness));
            }
        }

        Ok(default)
    }

    pub fn read_all<T>(&self, id: i16) -> Result<Vec<T>, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut parameter_values = Vec::new();
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id() == id {
                parameter_values.push(T::deserialize(&mut CdrDeserializer::new(
                    p.data(),
                    self.endianness,
                ))?);
            }
        }

        Ok(parameter_values)
    }
}

#[cfg(test)]
mod tests {
    use crate::cdr::parameter_list_deserialize::ParameterListDeserialize;

    use super::*;

    fn deserialize_pl_be<'de, T>(data: &'de [u8]) -> Result<T, std::io::Error>
    where
        T: ParameterListDeserialize<'de>,
    {
        let mut pl_deserializer = ParameterListDeserializer::new(data, CdrEndianness::BigEndian);
        ParameterListDeserialize::deserialize(&mut pl_deserializer)
    }

    fn deserialize_pl_le<'de, T>(data: &'de [u8]) -> Result<T, std::io::Error>
    where
        T: ParameterListDeserialize<'de>,
    {
        let mut pl_deserializer = ParameterListDeserializer::new(data, CdrEndianness::LittleEndian);
        ParameterListDeserialize::deserialize(&mut pl_deserializer)
    }

    #[test]
    fn deserialize_one_parameter_le() {
        #[derive(PartialEq, Debug)]
        struct OneParamData {
            value: u32,
        }

        impl<'de> ParameterListDeserialize<'de> for OneParamData {
            fn deserialize(
                pl_deserializer: &mut ParameterListDeserializer<'de>,
            ) -> Result<Self, std::io::Error> {
                let value = pl_deserializer.read(71)?;
                Ok(Self { value })
            }
        }

        let expected = OneParamData { value: 21 };

        assert_eq!(
            deserialize_pl_be::<OneParamData>(&[
                0, 71, 0, 4, // pid | Length (incl padding)
                0, 0, 0, 21, // CdrParameterType
                0, 1, 0, 0 // PID_SENTINEL
            ])
            .unwrap(),
            expected
        );
        assert_eq!(
            deserialize_pl_le::<OneParamData>(&[
                71, 0x00, 4, 0, // pid | Length (incl padding)
                21, 0, 0, 0, // CdrParameterType
                1, 0, 0, 0 // PID_SENTINEL
            ])
            .unwrap(),
            expected
        )
    }

    //     #[test]
    //     fn deserialize_one_parameter_be() {
    //         let data = &[
    //             0, 71, 0, 4, // pid | Length (incl padding)
    //             0, 0, 0, 21, // u32
    //         ][..];
    //         let expected: Parameter<71, u32> = Parameter::new(21);
    //         let mut deserializer = ParameterListDeserializer::<byteorder::BigEndian>::new(data);
    //         let result: Parameter<71, u32> =
    //             serde::Deserialize::deserialize(&mut deserializer).unwrap();
    //         assert_eq!(result, expected)
    //     }

    //     #[derive(Debug, PartialEq, serde::Deserialize)]
    //     struct PlInner {
    //         id: Parameter<71, u8>,
    //         n: Parameter<72, u8>,
    //     }
    //     #[test]
    //     fn deserialize_pl_simple() {
    //         let expected = PlInner {
    //             id: Parameter::new(21),
    //             n: Parameter::new(34),
    //         };
    //         let data = &[
    //             72, 0x00, 4, 0, // n | Length (incl padding)
    //             34, 0, 0, 0, // u8
    //             71, 0x00, 4, 0, // id | Length (incl padding)
    //             21, 0, 0, 0, // u8
    //             1, 0, 0, 0, // Sentinel
    //         ][..];
    //         let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
    //         let result: PlInner = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    //         assert_eq!(result, expected)
    //     }

    //     #[derive(Debug, PartialEq, serde::Deserialize)]
    //     struct PlWithDefault {
    //         id: Parameter<71, u8>,
    //         n: ParameterWithDefault<72, u8>,
    //     }
    //     #[test]
    //     fn deserialize_pl_with_default() {
    //         let expected = PlWithDefault {
    //             id: Parameter::new(21),
    //             n: ParameterWithDefault::new(0),
    //         };
    //         let data = &[
    //             71, 0x00, 4, 0, // n | Length (incl padding)
    //             21, 0, 0, 0, // u8
    //             1, 0, 0, 0, // Sentinel
    //         ][..];
    //         let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
    //         let result: PlWithDefault = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    //         assert_eq!(result, expected)
    //     }
    //     #[derive(Debug, PartialEq, serde::Deserialize)]
    //     struct PlWithList {
    //         id: Parameter<71, u8>,
    //         values: ParameterVector<93, u16>,
    //     }

    //     #[test]
    //     fn deserialize_pl_vec_le() {
    //         let expected = PlWithList {
    //             id: Parameter::new(21),
    //             values: ParameterVector::new(vec![34, 35]),
    //         };
    //         let data = &[
    //             71, 0x00, 4, 0, // id | Length (incl padding)
    //             21, 0, 0, 0, // u8
    //             93, 0x00, 4, 0, // values | Length (incl padding)
    //             34, 0, 0, 0, // u16
    //             93, 0x00, 4, 0, // values | Length (incl padding)
    //             35, 0, 0, 0, // u16
    //             1, 0, 0, 0, // Sentinel
    //         ][..];
    //         let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
    //         let result: PlWithList = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    //         assert_eq!(result, expected)
    //     }

    //     #[derive(Debug, PartialEq, serde::Deserialize)]
    //     struct PlOuter {
    //         outer: Parameter<2, u8>,
    //         inner: PlInner,
    //     }
    //     #[test]
    //     fn deserialize_dds_deserialize_compound() {
    //         let expected = PlOuter {
    //             outer: Parameter::new(7),
    //             inner: PlInner {
    //                 id: Parameter::new(21),
    //                 n: Parameter::new(34),
    //             },
    //         };
    //         let data = &[
    //             72, 0x00, 4, 0, // n | Length (incl padding)
    //             34, 0, 0, 0, // u8
    //             2, 0x00, 4, 0, // outer | Length (incl padding)
    //             7, 0, 0, 0, // u8
    //             71, 0x00, 4, 0, // id | Length (incl padding)
    //             21, 0, 0, 0, // u8
    //             1, 0, 0, 0, // Sentinel
    //         ][..];
    //         let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
    //         let result: PlOuter = serde::Deserialize::deserialize(&mut deserializer).unwrap();
    //         assert_eq!(result, expected)
    //     }
}
