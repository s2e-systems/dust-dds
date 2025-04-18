use crate::{
    implementation::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::error::{RtpsError, RtpsErrorKind},
    rtps_messages::types::ParameterId,
    xtypes::{
        deserialize::XTypesDeserialize,
        xcdr_deserializer::{Xcdr1BeDeserializer, Xcdr1LeDeserializer},
    },
};
use std::io::{BufRead, Read};

#[derive(Clone, Copy)]
enum CdrEndianness {
    BigEndian,
    LittleEndian,
}

type RepresentationIdentifier = [u8; 2];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];

struct Parameter<'de> {
    pid: ParameterId,
    data: &'de [u8],
}

impl<'de> Parameter<'de> {
    fn deserialize<T: XTypesDeserialize<'de>>(
        &self,
        endianness: CdrEndianness,
    ) -> Result<T, RtpsError> {
        Ok(match endianness {
            CdrEndianness::BigEndian => T::deserialize(&mut Xcdr1BeDeserializer::new(self.data))?,
            CdrEndianness::LittleEndian => {
                T::deserialize(&mut Xcdr1LeDeserializer::new(self.data))?
            }
        })
    }
}

struct ParameterIterator<'a> {
    reader: &'a [u8],
    endianness: CdrEndianness,
}

impl<'a> ParameterIterator<'a> {
    fn next(&mut self) -> Result<Option<Parameter<'a>>, RtpsError> {
        let mut pid = [0; 2];
        let mut length = [0; 2];
        self.reader.read_exact(&mut pid)?;
        self.reader.read_exact(&mut length)?;
        let pid = match self.endianness {
            CdrEndianness::BigEndian => ParameterId::from_be_bytes(pid),
            CdrEndianness::LittleEndian => ParameterId::from_le_bytes(pid),
        };
        let length = match self.endianness {
            CdrEndianness::BigEndian => ParameterId::from_be_bytes(length),
            CdrEndianness::LittleEndian => ParameterId::from_le_bytes(length),
        } as usize;
        if self.reader.len() < length {
            Err(RtpsError::new(
                RtpsErrorKind::NotEnoughData,
                "Not enough data to get parameter length".to_string(),
            ))
        } else if pid == PID_SENTINEL {
            Ok(None)
        } else {
            let data = &self.reader[..length];
            self.reader.consume(length);
            Ok(Some(Parameter { pid, data }))
        }
    }
}

pub struct ParameterListCdrDeserializer<'de> {
    bytes: &'de [u8],
    endianness: CdrEndianness,
}

impl<'de> ParameterListCdrDeserializer<'de> {
    pub fn new(bytes: &'de [u8]) -> Result<Self, RtpsError> {
        if bytes.len() < 4 {
            return Err(RtpsError::new(
                RtpsErrorKind::InvalidData,
                "Missing representation identifier".to_string(),
            ));
        }
        let endianness = match [bytes[0], bytes[1]] {
            PL_CDR_BE => CdrEndianness::BigEndian,
            PL_CDR_LE => CdrEndianness::LittleEndian,
            _ => Err(RtpsError::new(
                RtpsErrorKind::InvalidData,
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
    fn iter(&self) -> ParameterIterator<'de> {
        ParameterIterator {
            reader: self.bytes,
            endianness: self.endianness,
        }
    }

    pub fn read<T>(&self, pid: ParameterId) -> Result<T, RtpsError>
    where
        T: XTypesDeserialize<'de>,
    {
        let mut iterator = self.iter();
        while let Some(parameter) = iterator.next()? {
            if parameter.pid == pid {
                return parameter.deserialize(self.endianness);
            }
        }
        Err(RtpsError::new(
            crate::rtps::error::RtpsErrorKind::InvalidData,
            format!("Parameter with id {} not found", pid),
        ))
    }

    pub fn read_collection<T>(&self, pid: ParameterId) -> Result<Vec<T>, RtpsError>
    where
        T: XTypesDeserialize<'de>,
    {
        let mut collection = Vec::new();
        let mut iterator = self.iter();
        while let Some(parameter) = iterator.next()? {
            if parameter.pid == pid {
                collection.push(parameter.deserialize(self.endianness)?);
            }
        }
        Ok(collection)
    }

    pub fn read_with_default<T>(&self, pid: ParameterId, default: T) -> Result<T, RtpsError>
    where
        T: XTypesDeserialize<'de>,
    {
        let mut iterator = self.iter();
        while let Some(parameter) = iterator.next()? {
            if parameter.pid == pid {
                return parameter.deserialize(self.endianness);
            }
        }
        Ok(default)
    }
}
