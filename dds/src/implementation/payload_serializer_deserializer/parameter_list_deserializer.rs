use crate::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::error::RtpsError,
    xtypes::{
        deserialize::XTypesDeserialize,
        xcdr_deserializer::{Xcdr1BeDeserializer, Xcdr1LeDeserializer},
    },
};
use std::io::{BufRead, Read};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CdrEndianness {
    LittleEndian,
    BigEndian,
}

type RepresentationIdentifier = [u8; 2];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];

struct Parameter<'de> {
    id: i16,
    data: &'de [u8],
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
    pub fn new(bytes: &'de [u8]) -> Result<Self, RtpsError> {
        let endianness = match [bytes[0], bytes[1]] {
            PL_CDR_BE => CdrEndianness::BigEndian,
            PL_CDR_LE => CdrEndianness::LittleEndian,
            _ => Err(RtpsError::new(
                crate::rtps::error::RtpsErrorKind::InvalidData,
                "Unknownn representation identifier".to_string(),
            ))?,
        };

        Ok(Self {
            bytes: &bytes[4..],
            endianness,
        })
    }
}

impl<'de> ParameterListCdrDeserializer<'de> {
    pub fn read<T>(&self, id: i16) -> Result<T, RtpsError>
    where
        T: XTypesDeserialize<'de>,
    {
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id == id {
                return Ok(match self.endianness {
                    CdrEndianness::LittleEndian => {
                        T::deserialize(&mut Xcdr1LeDeserializer::new(p.data))?
                    }
                    CdrEndianness::BigEndian => {
                        T::deserialize(&mut Xcdr1BeDeserializer::new(p.data))?
                    }
                });
            }
        }

        Err(RtpsError::new(
            crate::rtps::error::RtpsErrorKind::InvalidData,
            format!("Parameter with id {} not found", id),
        ))
    }

    pub fn read_with_default<T>(&self, id: i16, default: T) -> Result<T, RtpsError>
    where
        T: XTypesDeserialize<'de>,
    {
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id == id {
                return Ok(match self.endianness {
                    CdrEndianness::LittleEndian => {
                        T::deserialize(&mut Xcdr1LeDeserializer::new(p.data))?
                    }
                    CdrEndianness::BigEndian => {
                        T::deserialize(&mut Xcdr1BeDeserializer::new(p.data))?
                    }
                });
            }
        }

        Ok(default)
    }

    pub fn read_collection<T>(&self, id: i16) -> Result<Vec<T>, RtpsError>
    where
        T: XTypesDeserialize<'de>,
    {
        let mut parameter_values = Vec::new();
        let mut bytes = self.bytes;
        let mut parameter_iterator = ParameterIterator::new(&mut bytes, self.endianness);
        while let Some(p) = parameter_iterator.next()? {
            if p.id == id {
                match self.endianness {
                    CdrEndianness::LittleEndian => parameter_values
                        .push(T::deserialize(&mut Xcdr1LeDeserializer::new(p.data))?),
                    CdrEndianness::BigEndian => parameter_values
                        .push(T::deserialize(&mut Xcdr1BeDeserializer::new(p.data))?),
                };
            }
        }

        Ok(parameter_values)
    }
}
