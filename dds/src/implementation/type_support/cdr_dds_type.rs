use std::io::{Read, Write};

use crate::{
    cdr::{
        deserialize::CdrDeserialize, deserializer::CdrDeserializer, endianness::CdrEndianness,
        parameter_list_serializer::ParameterListSerializer, serialize::CdrSerialize,
        serializer::CdrSerializer,
    },
    infrastructure::error::{DdsError, DdsResult},
    topic_definition::type_support::{DdsDeserialize, DdsSerializeData, DdsSerializedData},
};

type RepresentationIdentifier = [u8; 2];
type RepresentationOptions = [u8; 2];

const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];

impl<'de, T> DdsDeserialize<'de> for T
where
    T: CdrDeserialize<'de>,
{
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self> {
        let mut data_reader = serialized_data;
        let mut representation_identifier = [0u8, 0];
        data_reader
            .read_exact(&mut representation_identifier)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;

        let mut representation_option = [0u8, 0];
        data_reader
            .read_exact(&mut representation_option)
            .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))?;

        match representation_identifier {
            CDR_BE => {
                let mut deserializer = CdrDeserializer::new(data_reader, CdrEndianness::BigEndian);
                Ok(CdrDeserialize::deserialize(&mut deserializer)?)
            }
            PL_CDR_BE => {
                let mut deserializer = CdrDeserializer::new(data_reader, CdrEndianness::BigEndian);
                Ok(CdrDeserialize::deserialize(&mut deserializer)?)
            }
            CDR_LE => {
                let mut deserializer =
                    CdrDeserializer::new(data_reader, CdrEndianness::LittleEndian);
                Ok(CdrDeserialize::deserialize(&mut deserializer)?)
            }
            PL_CDR_LE => {
                let mut deserializer =
                    CdrDeserializer::new(data_reader, CdrEndianness::LittleEndian);
                Ok(CdrDeserialize::deserialize(&mut deserializer)?)
            }

            _ => Err(DdsError::Error(
                "Illegal representation identifier".to_string(),
            )),
        }
    }
}
