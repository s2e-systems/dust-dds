use std::io::{Read, Write};

use crate::{
    implementation::parameter_list_serde::{
        parameter_list_deserializer::ParameterListDeserializer,
        parameter_list_serializer::ParameterListSerializer,
    },
    infrastructure::error::{DdsError, DdsResult},
};

use byteorder::ByteOrder;
pub use dust_dds_derive::{DdsSerde, DdsType};

pub type RepresentationType = [u8; 2];
pub type RepresentationOptions = [u8; 2];

pub const CDR_BE: RepresentationType = [0x00, 0x00];
pub const CDR_LE: RepresentationType = [0x00, 0x01];
pub const PL_CDR_BE: RepresentationType = [0x00, 0x02];
pub const PL_CDR_LE: RepresentationType = [0x00, 0x03];
pub const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];

#[derive(Debug, PartialEq, Clone, Eq, serde::Serialize, serde::Deserialize)]
pub struct DdsSerializedKey(Vec<u8>);

impl From<&[u8]> for DdsSerializedKey {
    fn from(x: &[u8]) -> Self {
        Self(x.to_vec())
    }
}

impl From<Vec<u8>> for DdsSerializedKey {
    fn from(x: Vec<u8>) -> Self {
        Self(x)
    }
}

impl AsRef<[u8]> for DdsSerializedKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl DdsSerde for DdsSerializedKey {}

pub trait DdsType {
    fn type_name() -> &'static str;

    fn has_key() -> bool {
        false
    }

    fn get_serialized_key(&self) -> DdsSerializedKey {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for get_serialized_key")
        } else {
            DdsSerializedKey(vec![])
        }
    }

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

pub trait RepresentationFormat {
    const REPRESENTATION_IDENTIFIER: RepresentationType;
}

impl<T: serde::Serialize + DdsSerde> RepresentationFormat for T {
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;
}

pub trait DdsSerialize {
    const REPRESENTATION_IDENTIFIER: RepresentationType;
    fn dds_serialize<W: Write>(&self, mut writer: W) -> DdsResult<()> {
        let data = match Self::REPRESENTATION_IDENTIFIER {
            CDR_LE => {
                let mut d = vec![];
                let mut serializer =
                    cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut d);
                writer
                    .write_all(&CDR_LE)
                    .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
                self.dds_serialize_cdr(&mut serializer)?;
                d
            }
            PL_CDR_LE => {
                let mut d = vec![];
                let mut pl_serializer = ParameterListSerializer::new(&mut d);
                pl_serializer.serialize_payload_header()?;
                self.dds_serialize_parameter_list(&mut pl_serializer)?;
                pl_serializer.serialize_sentinel()?;
                d
            }
            _ => todo!(),
        };
        writer
            .write(data.as_slice())
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        Ok(())
    }

    fn dds_serialize_cdr<S: serde::Serializer>(&self, _serializer: S) -> DdsResult<()> {
        unimplemented!("dds_serialize_cdr not implemented for this type")
    }

    fn dds_serialize_parameter_list<W: Write>(
        &self,
        _serializer: &mut ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        unimplemented!("dds_serialize_parameter_list not implemented for this type")
    }
}

pub trait DdsDeserialize<'de>: Sized {
    fn dds_deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let mut representation_identifier: RepresentationType = [0, 0];
        buf.read_exact(&mut representation_identifier)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        let mut representation_option: RepresentationOptions = [0, 0];
        buf.read_exact(&mut representation_option)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;

        match representation_identifier {
            CDR_BE => {
                let mut deserializer =
                    cdr::Deserializer::<_, _, byteorder::BigEndian>::new(buf, cdr::Infinite);
                DdsDeserialize::dds_deserialize_cdr(&mut deserializer)
            }
            CDR_LE => {
                let mut deserializer =
                    cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(buf, cdr::Infinite);
                DdsDeserialize::dds_deserialize_cdr(&mut deserializer)
            }
            PL_CDR_BE => {
                let mut deserializer = ParameterListDeserializer::read(buf)?;
                DdsDeserialize::dds_deserialize_parameter_list::<byteorder::BigEndian>(
                    &mut deserializer,
                )
            }
            PL_CDR_LE => {
                let mut deserializer = ParameterListDeserializer::read(buf)?;
                DdsDeserialize::dds_deserialize_parameter_list::<byteorder::LittleEndian>(
                    &mut deserializer,
                )
            }
            _ => Err(DdsError::PreconditionNotMet(
                "Illegal representation identifier".to_string(),
            )),
        }
    }

    fn dds_deserialize_cdr<D: serde::Deserializer<'de>>(_deserializer: D) -> DdsResult<Self> {
        unimplemented!("dds_deserialize_cdr not implemented for this type")
    }

    fn dds_deserialize_parameter_list<E: ByteOrder>(
        _deserializer: &mut ParameterListDeserializer<'de, E>,
    ) -> DdsResult<Self> {
        unimplemented!("dds_deserialize_parameter_list not implemented for this type")
    }
}

pub trait DdsSerde {}

impl<Foo> DdsSerialize for Foo
where
    Foo: serde::Serialize + DdsSerde,
{
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;

    fn dds_serialize_cdr<S: serde::Serializer>(&self, serializer: S) -> DdsResult<()> {
        serde::Serialize::serialize(self, serializer)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        Ok(())
    }
}

impl<'de, Foo> DdsDeserialize<'de> for Foo
where
    Foo: serde::Deserialize<'de> + DdsSerde,
{
    fn dds_deserialize_cdr<D: serde::Deserializer<'de>>(deserializer: D) -> DdsResult<Self> {
        serde::Deserialize::deserialize(deserializer)
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::domain_participant_factory::DomainId,
        implementation::{
            data_representation_builtin_endpoints::parameter_id_values::{
                PID_DOMAIN_ID, PID_GROUP_ENTITYID,
            },
            rtps::types::{EntityId, EntityKey, BUILT_IN_READER_GROUP},
        },
    };

    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestDeserialize {
        remote_group_entity_id: EntityId,
        inner: TestDeserializeInner,
    }
    impl<'de> DdsDeserialize<'de> for TestDeserialize {
        fn dds_deserialize_parameter_list<E: ByteOrder>(
            deserializer: &mut ParameterListDeserializer<'de, E>,
        ) -> DdsResult<Self> {
            Ok(Self {
                remote_group_entity_id: deserializer.get(PID_GROUP_ENTITYID)?,
                inner: TestDeserializeInner::dds_deserialize_parameter_list(deserializer)?,
            })
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestDeserializeInner {
        domain_id: DomainId,
    }
    impl<'de> DdsDeserialize<'de> for TestDeserializeInner {
        fn dds_deserialize_parameter_list<E: ByteOrder>(
            deserializer: &mut ParameterListDeserializer<'de, E>,
        ) -> DdsResult<Self> {
            Ok(Self {
                domain_id: deserializer.get(PID_DOMAIN_ID)?,
            })
        }
    }

    #[test]
    fn dds_deserialize_simple() {
        let expected = TestDeserialize {
            remote_group_entity_id: EntityId::new(
                EntityKey::new([21, 22, 23]),
                BUILT_IN_READER_GROUP,
            ),
            inner: TestDeserializeInner { domain_id: 2 },
        };

        let data = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE | OPTIONS
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x02, 0x00, 0x00, 0x00, // DomainId
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        let result: TestDeserialize =
            DdsDeserialize::dds_deserialize(&mut data.as_slice()).unwrap();
        assert_eq!(result, expected);
    }

    struct TestSerialize {
        remote_group_entity_id: EntityId,
        inner: TestSerializeInner,
    }

    struct TestSerializeInner {
        domain_id: DomainId,
    }

    impl DdsSerialize for TestSerialize {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

        fn dds_serialize_parameter_list<W: Write>(
            &self,
            serializer: &mut ParameterListSerializer<W>,
        ) -> DdsResult<()> {
            serializer.serialize_parameter(PID_GROUP_ENTITYID, &self.remote_group_entity_id)?;

            self.inner.dds_serialize_parameter_list(serializer)
        }

        fn dds_serialize<W: Write>(&self, mut writer: W) -> DdsResult<()> {
            let mut data = vec![];
            let mut pl_serializer = ParameterListSerializer::new(&mut data);
            pl_serializer.serialize_payload_header().unwrap();
            self.dds_serialize_parameter_list(&mut pl_serializer)
                .unwrap();
            pl_serializer.serialize_sentinel().unwrap();
            writer
                .write(data.as_slice())
                .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
            Ok(())
        }
    }

    impl DdsSerialize for TestSerializeInner {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

        fn dds_serialize_parameter_list<W: Write>(
            &self,
            serializer: &mut ParameterListSerializer<W>,
        ) -> DdsResult<()> {
            serializer.serialize_parameter(PID_DOMAIN_ID, &self.domain_id)
        }
    }

    #[test]
    fn dds_serialize_simple() {
        let data = TestSerialize {
            remote_group_entity_id: EntityId::new(
                EntityKey::new([21, 22, 23]),
                BUILT_IN_READER_GROUP,
            ),
            inner: TestSerializeInner { domain_id: 2 },
        };

        let mut writer = Vec::<u8>::new();
        data.dds_serialize(&mut writer).unwrap();

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE | OPTIONS
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x0f, 0x00, 0x04, 0x00, // PID_DOMAIN_ID, Length: 4
            0x02, 0x00, 0x00, 0x00, // DomainId
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(writer, expected);
    }
}
