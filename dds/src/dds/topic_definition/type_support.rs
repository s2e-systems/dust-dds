use std::io::{Read, Write};

use crate::{
    implementation::parameter_list_serde::{
        serde_parameter_list_deserializer::ParameterListDeserializer,
        serde_parameter_list_serializer::ParameterListSerializer,
    },
    infrastructure::error::{
        DdsError::{self, PreconditionNotMet},
        DdsResult,
    },
};

pub use dust_dds_derive::DdsType;

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

impl DdsType for DdsSerializedKey {
    fn has_key() -> bool {
        false
    }
}

pub trait DdsType {
    const REPRESENTATION_IDENTIFIER: RepresentationType = CDR_LE;

    fn has_key() -> bool;

    fn set_key_fields_from_serialized_key(&mut self, _key: &DdsSerializedKey) -> DdsResult<()> {
        if Self::has_key() {
            unimplemented!("DdsType with key must provide an implementation for set_key_fields_from_serialized_key")
        }
        Ok(())
    }
}

pub trait DdsKey {
    // Serde trait bounds placed here since there is no easy way to express this bound
    // without specifying the lifetime 'a which makes the bounds on usage very complicated
    type BorrowedKeyHolder<'a>: serde::Serialize
    where
        Self: 'a;
    type OwningKeyHolder: for<'de> serde::Deserialize<'de>;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_>;

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder);
}

pub trait DdsKeyDeserialize {
    type OwningKeyHolder;
}

pub fn dds_deserialize_key<'de, T>(data: &'de [u8]) -> DdsResult<T::OwningKeyHolder>
where
    T: DdsKeyDeserialize,
    T::OwningKeyHolder: serde::Deserialize<'de>,
{
    let mut deserializer =
        cdr::Deserializer::<_, _, byteorder::BigEndian>::new(data, cdr::Infinite);
    serde::Deserialize::deserialize(&mut deserializer)
        .map_err(|err| DdsError::PreconditionNotMet(err.to_string()))
}

pub fn dds_serialize_to_bytes<T>(value: &T) -> DdsResult<Vec<u8>>
where
    T: serde::Serialize + DdsType,
{
    let mut writer = vec![];
    match T::REPRESENTATION_IDENTIFIER {
        CDR_BE => {
            writer
                .write_all(&CDR_BE)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            writer
                .write_all(&REPRESENTATION_OPTIONS)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            let mut serializer = cdr::ser::Serializer::<_, byteorder::BigEndian>::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
        }
        CDR_LE => {
            writer
                .write_all(&CDR_LE)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            writer
                .write_all(&REPRESENTATION_OPTIONS)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            let mut serializer =
                cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
        }
        PL_CDR_BE => {
            writer
                .write_all(&PL_CDR_BE)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            writer
                .write_all(&REPRESENTATION_OPTIONS)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            let mut serializer =
                ParameterListSerializer::<_, byteorder::BigEndian>::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
        }
        PL_CDR_LE => {
            writer
                .write_all(&PL_CDR_LE)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            writer
                .write_all(&REPRESENTATION_OPTIONS)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            let mut serializer =
                ParameterListSerializer::<_, byteorder::LittleEndian>::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
        }
        _ => todo!(),
    };
    Ok(writer)
}

pub fn dds_deserialize_from_bytes<'de, T>(mut data: &'de [u8]) -> DdsResult<T>
where
    T: serde::Deserialize<'de>,
{
    let mut representation_identifier: RepresentationType = [0, 0];
    data.read_exact(&mut representation_identifier)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;

    let mut representation_option: RepresentationOptions = [0, 0];
    data.read_exact(&mut representation_option)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;

    match representation_identifier {
        CDR_BE => {
            let mut deserializer =
                cdr::Deserializer::<_, _, byteorder::BigEndian>::new(data, cdr::Infinite);
            serde::Deserialize::deserialize(&mut deserializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))
        }
        CDR_LE => {
            let mut deserializer =
                cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(data, cdr::Infinite);
            serde::Deserialize::deserialize(&mut deserializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))
        }
        PL_CDR_BE => {
            let mut deserializer = ParameterListDeserializer::<byteorder::BigEndian>::new(data);
            serde::Deserialize::deserialize(&mut deserializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))
        }
        PL_CDR_LE => {
            let mut deserializer = ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
            serde::Deserialize::deserialize(&mut deserializer)
                .map_err(|err| PreconditionNotMet(err.to_string()))
        }
        _ => Err(PreconditionNotMet(
            "Illegal representation identifier".to_string(),
        )),
    }
}

pub fn dds_serialize_key_to_bytes<T>(value: &T) -> DdsResult<DdsSerializedKey>
where
    T: DdsKey,
{
    let mut writer = vec![];

    let mut serializer = cdr::ser::Serializer::<_, byteorder::BigEndian>::new(&mut writer);
    let key = value.get_key();
    serde::Serialize::serialize(&key, &mut serializer)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;
    Ok(writer.into())
}
