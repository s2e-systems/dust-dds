use std::io::{Read, Write};

use crate::{
    cdr::{
        deserialize::CdrDeserialize,
        deserializer::CdrDeserializer,
        endianness::CdrEndianness,
        representation::{CdrRepresentation, CdrRepresentationKind},
        serialize::CdrSerialize,
        serializer::CdrSerializer,
    },
    implementation::{
        data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
        parameter_list_serde::parameter::Parameter,
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

impl<T> DdsSerializeData for T
where
    T: CdrSerialize + CdrRepresentation,
{
    fn serialize_data(&self) -> DdsResult<DdsSerializedData> {
        let mut writer = Vec::new();
        match T::REPRESENTATION {
            CdrRepresentationKind::CdrLe => {
                writer
                    .write_all(&CDR_LE)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                let mut serializer = CdrSerializer::new(&mut writer, CdrEndianness::LittleEndian);
                self.serialize(&mut serializer)?;
            }
            CdrRepresentationKind::CdrBe => {
                writer
                    .write_all(&CDR_BE)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                let mut serializer = CdrSerializer::new(&mut writer, CdrEndianness::BigEndian);
                self.serialize(&mut serializer)?;
            }
            CdrRepresentationKind::PlCdrBe => {
                writer
                    .write_all(&PL_CDR_BE)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| DdsError::Error(err.to_string()))?;

                let mut serializer = CdrSerializer::new(&mut writer, CdrEndianness::BigEndian);
                self.serialize(&mut serializer)?;
                Parameter::<PID_SENTINEL, ()>::new(()).serialize(&mut serializer)?;
            }
            CdrRepresentationKind::PlCdrLe => {
                writer
                    .write_all(&PL_CDR_LE)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                let mut serializer = CdrSerializer::new(&mut writer, CdrEndianness::LittleEndian);
                self.serialize(&mut serializer)?;
                Parameter::<PID_SENTINEL, ()>::new(()).serialize(&mut serializer)?;
            }
        };
        Ok(writer.into())
    }
}

impl<'de, T> DdsDeserialize<'de> for T
where
    T: CdrDeserialize<'de> + CdrRepresentation,
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
