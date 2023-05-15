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

pub trait DdsSerde {}

impl<T: serde::Serialize + DdsSerde> RepresentationFormat for T {
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;
}

pub trait DdsSerialize: serde::Serialize {}

impl<Foo> DdsSerialize for Foo where Foo: serde::Serialize {}

pub trait DdsDeserialize<'de>: serde::Deserialize<'de> + Sized {}

impl<'de, Foo> DdsDeserialize<'de> for Foo where Foo: serde::Deserialize<'de> {}
