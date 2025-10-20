use crate::{
    dcps::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::error::RtpsError,
    rtps_messages::{
        overall_structure::{BufRead, Read},
        types::ParameterId,
    },
    xtypes::{
        data_representation::DataKind,
        deserializer::XTypesDeserializer,
        dynamic_type::{DynamicData, DynamicType},
        error::XTypesResult,
    },
};
use alloc::vec::Vec;

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
    fn deserialize(&self, endianness: CdrEndianness) -> Result<DataKind, RtpsError> {
        todo!()
        // Ok(match endianness {
        //     CdrEndianness::BigEndian => T::deserialize(&mut Xcdr1BeDeserializer::new(self.data))?,
        //     CdrEndianness::LittleEndian => {
        //         T::deserialize(&mut Xcdr1LeDeserializer::new(self.data))?
        //     }
        // })
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
            Err(RtpsError::NotEnoughData)
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
            return Err(RtpsError::InvalidData);
        }
        let endianness = match [bytes[0], bytes[1]] {
            PL_CDR_BE => CdrEndianness::BigEndian,
            PL_CDR_LE => CdrEndianness::LittleEndian,
            _ => Err(RtpsError::InvalidData)?,
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

    pub fn read(&self, pid: ParameterId) -> Result<DataKind, RtpsError> {
        todo!()
        // let mut iterator = self.iter();
        // while let Some(parameter) = iterator.next()? {
        //     if parameter.pid == pid {
        //         return parameter.deserialize(self.endianness);
        //     }
        // }
        // Err(RtpsError::ParameterNotFound)
    }

    pub fn read_collection(&self, pid: ParameterId) -> Result<Vec<DataKind>, RtpsError> {
        // let mut collection = Vec::new();
        // let mut iterator = self.iter();
        // while let Some(parameter) = iterator.next()? {
        //     if parameter.pid == pid {
        //         collection.push(parameter.deserialize(self.endianness)?);
        //     }
        // }
        // Ok(collection)
        todo!()
    }

    pub fn read_with_default(
        &self,
        pid: ParameterId,
        default: DataKind,
    ) -> Result<DataKind, RtpsError> {
        todo!()
        // let mut iterator = self.iter();
        // while let Some(parameter) = iterator.next()? {
        //     if parameter.pid == pid {
        //         return parameter.deserialize(self.endianness);
        //     }
        // }
        // Ok(default)
    }
}

pub struct PlCdrDeserializer {}

impl<'de> XTypesDeserializer<'de> for &mut PlCdrDeserializer {
    fn deserialize_final_struct(
        &mut self,
        v: &mut DynamicData,
    ) -> Result<(), super::error::XTypesError> {
        todo!()
        // Ok(self)
    }

    fn deserialize_appendable_struct(&mut self) -> Result<(), super::error::XTypesError> {
        todo!()
        // Ok(self)
    }

    fn deserialize_mutable_struct(&mut self) -> Result<(), super::error::XTypesError> {
        todo!()
        // Ok(self)
    }

    fn deserialize_array(&mut self) -> Result<(), super::error::XTypesError> {
        todo!()
        // Ok(self)
    }

    fn deserialize_sequence(&mut self) -> Result<(), super::error::XTypesError> {
        todo!()
        // Ok(self)
    }

    fn deserialize_boolean(&mut self) -> Result<bool, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int8(&mut self) -> Result<i8, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int16(&mut self) -> Result<i16, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int32(&mut self) -> Result<i32, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int64(&mut self) -> Result<i64, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint8(&mut self) -> Result<u8, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint16(&mut self) -> Result<u16, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint32(&mut self) -> Result<u32, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint64(&mut self) -> Result<u64, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_float32(&mut self) -> Result<f32, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_float64(&mut self) -> Result<f64, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_char8(&mut self) -> Result<char, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_string(&mut self) -> Result<&'de str, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_byte_sequence(&mut self) -> Result<&'de [u8], super::error::XTypesError> {
        todo!()
    }

    fn deserialize_byte_array<const N: usize>(
        &mut self,
    ) -> Result<&'de [u8; N], super::error::XTypesError> {
        todo!()
    }
    fn deserialize_data_kind(&mut self, dynamic_type: &DynamicType) -> XTypesResult<DataKind> {
        todo!()
    }
}
