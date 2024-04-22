use std::io::{BufRead, Read};

use crate::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    serialized_payload::{
        cdr::deserialize::CdrDeserialize, parameter_list::deserializer::ParameterListDeserializer,
    },
};

use super::{cdr_deserializer::ClassicCdrDeserializer, endianness::CdrEndianness};

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
                "Not enough data to get parameter length".to_string(),
            ))
        } else if id == PID_SENTINEL {
            Ok(None)
        } else {
            let data = &self.reader[..length];
            self.reader.consume(length);
            Ok(Some(Parameter { id, data }))
        }
    }
}

pub struct ParameterListCdrDeserializer<'de> {
    bytes: &'de [u8],
    endianness: CdrEndianness,
}

impl<'de> ParameterListCdrDeserializer<'de> {
    pub fn new(bytes: &'de [u8], endianness: CdrEndianness) -> Self {
        Self { bytes, endianness }
    }
}

impl<'de> ParameterListDeserializer<'de> for ParameterListCdrDeserializer<'de> {
    fn read<T>(&self, id: i16) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id() == id {
                return T::deserialize(&mut ClassicCdrDeserializer::new(p.data(), self.endianness));
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Parameter with id {} not found", id),
        ))
    }

    fn read_with_default<T>(&self, id: i16, default: T) -> Result<T, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id() == id {
                return T::deserialize(&mut ClassicCdrDeserializer::new(p.data(), self.endianness));
            }
        }

        Ok(default)
    }

    fn read_collection<T>(&self, id: i16) -> Result<Vec<T>, std::io::Error>
    where
        T: CdrDeserialize<'de>,
    {
        let mut parameter_values = Vec::new();
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id() == id {
                parameter_values.push(T::deserialize(&mut ClassicCdrDeserializer::new(
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
    use crate::serialized_payload::parameter_list::deserialize::ParameterListDeserialize;

    use super::*;

    fn deserialize_pl_be<'de, T>(data: &'de [u8]) -> Result<T, std::io::Error>
    where
        T: ParameterListDeserialize<'de>,
    {
        let mut pl_deserializer = ParameterListCdrDeserializer::new(data, CdrEndianness::BigEndian);
        ParameterListDeserialize::deserialize(&mut pl_deserializer)
    }

    fn deserialize_pl_le<'de, T>(data: &'de [u8]) -> Result<T, std::io::Error>
    where
        T: ParameterListDeserialize<'de>,
    {
        let mut pl_deserializer =
            ParameterListCdrDeserializer::new(data, CdrEndianness::LittleEndian);
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
                pl_deserializer: &mut impl ParameterListDeserializer<'de>,
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
}
