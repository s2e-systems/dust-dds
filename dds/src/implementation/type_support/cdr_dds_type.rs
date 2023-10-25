use std::io::Write;

use crate::{
    implementation::{
        data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
        parameter_list_serde::parameter::Parameter,
    },
    infrastructure::error::{DdsError, DdsResult},
    topic_definition::{
        cdr_type::{CdrRepresentation, CdrRepresentationKind, CdrSerialize},
        type_support::{DdsSerializeData, DdsSerializedData},
    },
};

use super::cdr_serializer::CdrDataSerializer;

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
                let mut serializer =
                    CdrDataSerializer::<_, byteorder::LittleEndian>::new(&mut writer);
                self.serialize(&mut serializer)?;
            }
            CdrRepresentationKind::CdrBe => {
                writer
                    .write_all(&CDR_BE)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                let mut serializer = CdrDataSerializer::<_, byteorder::BigEndian>::new(&mut writer);
                self.serialize(&mut serializer)?;
            }
            CdrRepresentationKind::PlCdrBe => {
                writer
                    .write_all(&PL_CDR_BE)
                    .map_err(|err| DdsError::Error(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| DdsError::Error(err.to_string()))?;

                let mut serializer = CdrDataSerializer::<_, byteorder::BigEndian>::new(&mut writer);
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
                let mut serializer =
                    CdrDataSerializer::<_, byteorder::LittleEndian>::new(&mut writer);
                self.serialize(&mut serializer)?;
                Parameter::<PID_SENTINEL, ()>::new(()).serialize(&mut serializer)?;
            }
        };
        Ok(writer.into())
    }
}
