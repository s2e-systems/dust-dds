use std::io::Write;

use crate::infrastructure::error::{DdsError, DdsResult};

use cdr::Encapsulation;
pub use dust_dds_derive::{DdsSerde, DdsType};

use super::pl_serializer;

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
        let sentinel = [1u8, 0, 0, 0];
        let data = match Self::REPRESENTATION_IDENTIFIER {
            CDR_LE => cdr::serialize::<_, _, cdr::CdrLe>(self, cdr::Infinite)
                .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?,
            PL_CDR_LE => {
                let mut d = vec![];
                let mut pl_serializer =
                    pl_serializer::Serializer::<_, byteorder::LittleEndian>::new(&mut d);
                serde::Serialize::serialize(&cdr::PlCdrLe::id(), &mut pl_serializer).unwrap();
                serde::Serialize::serialize(&cdr::PlCdrLe::option(), &mut pl_serializer).unwrap();
                serde::Serialize::serialize(self, &mut pl_serializer).unwrap();
                serde::Serialize::serialize(&sentinel, &mut pl_serializer).unwrap();
                d
            }
            _ => todo!(),
        };
        writer
            .write(data.as_slice())
            .map_err(|e| DdsError::PreconditionNotMet(e.to_string()))?;
        Ok(())
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

    use crate::implementation::{
        data_representation_builtin_endpoints::parameter_id_values::PID_GROUP_ENTITYID,
        rtps::types::{EntityId, EntityKey, BUILT_IN_READER_GROUP},
        rtps_udp_psm::mapping_traits::NumberOfBytes,
    };

    use super::*;

    struct TestBuiltIn {
        remote_group_entity_id: EntityId,
    }
    impl TestBuiltIn {
        fn new(remote_group_entity_id: EntityId) -> Self {
            Self {
                remote_group_entity_id,
            }
        }
    }
    impl serde::Serialize for TestBuiltIn {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut bytes = vec![];
            let mut cdr_serializer =
                cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut bytes);
            serde::Serialize::serialize(&PID_GROUP_ENTITYID, &mut cdr_serializer).unwrap();
            serde::Serialize::serialize(
                &(self.remote_group_entity_id.number_of_bytes() as u16),
                &mut cdr_serializer,
            )
            .unwrap();
            serde::Serialize::serialize(&self.remote_group_entity_id, &mut cdr_serializer).unwrap();
            serializer.serialize_bytes(&bytes)
        }
    }
    impl DdsSerialize for TestBuiltIn {
        const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;
    }

    #[test]
    fn serialize_all_default() {
        let data = TestBuiltIn::new(EntityId::new(
            EntityKey::new([21, 22, 23]),
            BUILT_IN_READER_GROUP,
        ));

        let mut writer = Vec::<u8>::new();
        data.dds_serialize(&mut writer).unwrap();

        let expected = vec![
            0x00, 0x03, 0x00, 0x00, // PL_CDR_LE | OPTIONS
            0x53, 0x00, 4, 0, //PID_GROUP_ENTITYID
            21, 22, 23, 0xc9, // u8[3], u8
            0x01, 0x00, 0x00, 0x00, // PID_SENTINEL, length
        ];
        assert_eq!(writer, expected);
    }
}
