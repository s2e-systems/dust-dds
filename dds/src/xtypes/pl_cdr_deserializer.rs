use crate::{
    dcps::data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::error::RtpsError,
    rtps_messages::{
        overall_structure::{BufRead, Read},
        types::ParameterId,
    },
    xtypes::{
        data_representation::DataKind,
        deserialize::XTypesDeserialize,
        deserializer::XTypesDeserializer,
        dynamic_type::DynamicType,
        error::XTypesResult,
        xcdr_deserializer::{Xcdr1BeDeserializer, Xcdr1LeDeserializer},
    },
};
use alloc::vec::Vec;

use super::deserializer::{
    DeserializeAppendableStruct, DeserializeArray, DeserializeFinalStruct,
    DeserializeMutableStruct, DeserializeSequence,
};

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
        Err(RtpsError::ParameterNotFound)
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

pub struct PlCdrDeserializer {}

impl<'de> XTypesDeserializer<'de> for &mut PlCdrDeserializer {
    fn deserialize_final_struct(
        self,
    ) -> Result<impl DeserializeFinalStruct<'de>, super::error::XTypesError> {
        Ok(self)
    }

    fn deserialize_appendable_struct(
        self,
    ) -> Result<impl DeserializeAppendableStruct<'de>, super::error::XTypesError> {
        Ok(self)
    }

    fn deserialize_mutable_struct(
        self,
    ) -> Result<impl DeserializeMutableStruct<'de>, super::error::XTypesError> {
        Ok(self)
    }

    fn deserialize_array(self) -> Result<impl DeserializeArray<'de>, super::error::XTypesError> {
        Ok(self)
    }

    fn deserialize_sequence(
        self,
    ) -> Result<impl DeserializeSequence<'de>, super::error::XTypesError> {
        Ok(self)
    }

    fn deserialize_boolean(self) -> Result<bool, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int8(self) -> Result<i8, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int16(self) -> Result<i16, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int32(self) -> Result<i32, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_int64(self) -> Result<i64, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint8(self) -> Result<u8, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint16(self) -> Result<u16, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint32(self) -> Result<u32, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_uint64(self) -> Result<u64, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_float32(self) -> Result<f32, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_float64(self) -> Result<f64, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_char8(self) -> Result<char, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_string(self) -> Result<&'de str, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_byte_sequence(self) -> Result<&'de [u8], super::error::XTypesError> {
        todo!()
    }

    fn deserialize_byte_array<const N: usize>(
        self,
    ) -> Result<&'de [u8; N], super::error::XTypesError> {
        todo!()
    }
    fn deserialize_data_kind(self, dynamic_type: &DynamicType) -> XTypesResult<DataKind> {
        todo!()
    }
}

impl<'de> DeserializeFinalStruct<'de> for &mut PlCdrDeserializer {
    fn deserialize_field(
        &mut self,
        descriptor: &super::dynamic_type::MemberDescriptor,
    ) -> Result<super::data_representation::DataKind, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_optional_field(
        &mut self,
        descriptor: &super::dynamic_type::MemberDescriptor,
    ) -> Result<Option<super::data_representation::DataKind>, super::error::XTypesError> {
        todo!()
    }
}

impl<'de> DeserializeAppendableStruct<'de> for &mut PlCdrDeserializer {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        name: &str,
    ) -> Result<T, super::error::XTypesError> {
        todo!()
    }
}

impl<'de> DeserializeMutableStruct<'de> for &mut PlCdrDeserializer {
    fn deserialize_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u32,
        name: &str,
    ) -> Result<T, super::error::XTypesError> {
        todo!()
    }

    fn deserialize_optional_field<T: XTypesDeserialize<'de>>(
        &mut self,
        pid: u32,
        name: &str,
    ) -> Result<Option<T>, super::error::XTypesError> {
        todo!()
    }
}

impl<'de> DeserializeArray<'de> for &mut PlCdrDeserializer {
    fn deserialize_element<T: XTypesDeserialize<'de>>(
        &mut self,
    ) -> Result<T, super::error::XTypesError> {
        todo!()
    }
}

impl<'de> DeserializeSequence<'de> for &mut PlCdrDeserializer {
    fn len(&self) -> usize {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn deserialize_element<T: XTypesDeserialize<'de>>(
        &mut self,
    ) -> Result<T, super::error::XTypesError> {
        todo!()
    }
}
