use std::io::Write;

use crate::{
    implementation::parameter_list_serde::parameter_list_serializer::ParameterListSerializer,
    infrastructure::error::{DdsError, DdsResult},
};

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

pub trait DdsSerialize: serde::Serialize {
    const REPRESENTATION_IDENTIFIER: RepresentationType;
    fn dds_serialize<W: Write>(&self, mut writer: W) -> DdsResult<()> {
        let data = match Self::REPRESENTATION_IDENTIFIER {
            CDR_LE => cdr::serialize::<_, _, cdr::CdrLe>(self, cdr::Infinite)
                .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?,
            PL_CDR_LE => {
                let mut d = vec![];
                let mut pl_serializer = ParameterListSerializer::new(&mut d);
                pl_serializer.serialize_payload_header().unwrap();
                self.dds_serialize_parameter_list(&mut pl_serializer).unwrap();
                pl_serializer.serialize_sentinel().unwrap();
                d
            }
            _ => todo!(),
        };
        writer
            .write(data.as_slice())
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        Ok(())
    }

    fn dds_serialize_parameter_list<W: Write>(
        &self,
        _serializer: &mut ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        unimplemented!("parameter_list serialization not implemented for this type")
    }
}

pub trait DdsDeserialize<'de>: Sized {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self>;
}

pub trait DdsSerde {}

impl<Foo> DdsSerialize for Foo
where
    Foo: serde::Serialize + DdsSerde,
{
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;
}

impl<'de, Foo> DdsDeserialize<'de> for Foo
where
    Foo: serde::Deserialize<'de> + DdsSerde,
{
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        cdr::deserialize(buf).map_err(|e| DdsError::PreconditionNotMet(e.to_string()))
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

    #[derive(serde::Serialize)]
    struct TestBuiltIn {
        remote_group_entity_id: EntityId,
        inner: TestBuiltInInner,
    }

    #[derive(serde::Serialize)]
    struct TestBuiltInInner {
        domain_id: DomainId,
    }

    impl DdsSerialize for TestBuiltIn {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

        fn dds_serialize_parameter_list<W: Write>(
            &self,
            serializer: &mut ParameterListSerializer<W>,
        ) -> DdsResult<()> {
            serializer.serialize_parameter(PID_GROUP_ENTITYID, &self.remote_group_entity_id)?;

            self.inner.dds_serialize_parameter_list(serializer)
        }
    }

    impl DdsSerialize for TestBuiltInInner {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

        fn dds_serialize_parameter_list<W: Write>(
            &self,
            serializer: &mut ParameterListSerializer<W>,
        ) -> DdsResult<()> {
            serializer.serialize_parameter(PID_DOMAIN_ID, &self.domain_id)
        }
    }

    #[test]
    fn serialize_all_default() {
        let data = TestBuiltIn {
            remote_group_entity_id: EntityId::new(
                EntityKey::new([21, 22, 23]),
                BUILT_IN_READER_GROUP,
            ),
            inner: TestBuiltInInner { domain_id: 2 },
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
