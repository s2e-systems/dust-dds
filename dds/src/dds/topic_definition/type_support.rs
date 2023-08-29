use std::io::{Read, Write};

use crate::{
    implementation::parameter_list_serde::{
        serde_parameter_list_deserializer::ParameterListDeserializer,
        serde_parameter_list_serializer::ParameterListSerializer,
    },
    infrastructure::error::{DdsError::PreconditionNotMet, DdsResult},
};

pub use dust_dds_derive::{DdsGetKey, DdsHasKey, DdsRepresentation, DdsSetKeyFields, DdsType};

pub enum Representation {
    CdrLe,
    CdrBe,
    PlCdrBe,
    PlCdrLe,
    Xml,
    Custom,
}

type RepresentationIdentifier = [u8; 2];
type RepresentationOptions = [u8; 2];

const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
const XML: RepresentationIdentifier = [0x00, 0x04];
const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];

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

pub trait DdsType: DdsRepresentation + DdsHasKey + DdsGetKey + DdsSetKeyFields {}

pub trait DdsHasKey {
    const HAS_KEY: bool;
}

pub trait DdsRepresentation {
    const REPRESENTATION: Representation;

    fn to_bytes(&self, _writer: impl std::io::Write) -> DdsResult<()> {
        unimplemented!("Custom type conversion to bytes is not implemented")
    }

    fn from_bytes(_bytes: &[u8]) -> DdsResult<Self>
    where
        Self: Sized,
    {
        unimplemented!("Custom type conversion from bytes is not implemented")
    }
}

pub trait DdsGetKey {
    // Serde trait bounds placed here since there is no easy way to express this bound
    // without specifying the lifetime 'a which makes the bounds on usage very complicated
    type BorrowedKeyHolder<'a>: serde::Serialize
    where
        Self: 'a;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_>;
}

pub trait DdsSetKeyFields {
    type OwningKeyHolder: for<'de> serde::Deserialize<'de>;

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder);
}

macro_rules! implement_dds_get_key_for_built_in_type {
    ($t:ty) => {
        impl DdsGetKey for $t {
            type BorrowedKeyHolder<'a> = $t;

            fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
                *self
            }
        }

        impl DdsSetKeyFields for $t {
            type OwningKeyHolder = $t;

            fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
                *self = key_holder;
            }
        }
    };
}

implement_dds_get_key_for_built_in_type!(bool);
implement_dds_get_key_for_built_in_type!(char);
implement_dds_get_key_for_built_in_type!(u8);
implement_dds_get_key_for_built_in_type!(i8);
implement_dds_get_key_for_built_in_type!(u16);
implement_dds_get_key_for_built_in_type!(i16);
implement_dds_get_key_for_built_in_type!(u32);
implement_dds_get_key_for_built_in_type!(i32);
implement_dds_get_key_for_built_in_type!(u64);
implement_dds_get_key_for_built_in_type!(i64);
implement_dds_get_key_for_built_in_type!(usize);
implement_dds_get_key_for_built_in_type!(isize);
implement_dds_get_key_for_built_in_type!(f32);
implement_dds_get_key_for_built_in_type!(f64);

impl<T> DdsGetKey for Vec<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type BorrowedKeyHolder<'a> = &'a Vec<T> where T:'a;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self
    }
}

impl<T> DdsSetKeyFields for Vec<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type OwningKeyHolder = Vec<T>;

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
        *self = key_holder;
    }
}

impl DdsGetKey for String {
    type BorrowedKeyHolder<'a> = &'a str;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self
    }
}

impl DdsSetKeyFields for String {
    type OwningKeyHolder = String;

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
        *self = key_holder;
    }
}

impl<const N: usize, T> DdsGetKey for [T; N]
where
    [T; N]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type BorrowedKeyHolder<'a> = &'a [T; N] where T:'a;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self
    }
}

impl<const N: usize, T> DdsSetKeyFields for [T; N]
where
    [T; N]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type OwningKeyHolder = [T; N];

    fn set_key_from_holder(&mut self, key_holder: Self::OwningKeyHolder) {
        *self = key_holder;
    }
}

pub fn dds_serialize_to_bytes<T>(value: &T) -> DdsResult<Vec<u8>>
where
    T: serde::Serialize + DdsRepresentation,
{
    let mut writer = vec![];
    match T::REPRESENTATION {
        Representation::CdrLe => {
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
        Representation::CdrBe => {
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
        Representation::PlCdrBe => {
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
        Representation::PlCdrLe => {
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
        Representation::Xml => {
            writer
                .write_all(&PL_CDR_LE)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            writer
                .write_all(&REPRESENTATION_OPTIONS)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;
            let mut serializer = serde_xml_rs::Serializer::new(&mut writer);
            serde::Serialize::serialize(value, &mut serializer)
                .map_err(|e| PreconditionNotMet(e.to_string()))?;
        }
        Representation::Custom => {
            T::to_bytes(value, &mut writer)?;
        }
    };
    Ok(writer)
}

pub fn dds_deserialize_from_bytes<'de, T>(mut data: &'de [u8]) -> DdsResult<T>
where
    T: serde::Deserialize<'de> + DdsRepresentation,
{
    match T::REPRESENTATION {
        Representation::Custom => {
            todo!()
        }
        _ => {
            let mut REPRESENTATION = [0u8, 0];
            data.read_exact(&mut REPRESENTATION)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;

            let mut representation_option = [0u8, 0];
            data.read_exact(&mut representation_option)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;

            match REPRESENTATION {
                CDR_BE => {
                    let mut deserializer =
                        cdr::Deserializer::<_, _, byteorder::BigEndian>::new(data, cdr::Infinite);
                    serde::Deserialize::deserialize(&mut deserializer)
                        .map_err(|err| PreconditionNotMet(err.to_string()))
                }
                CDR_LE => {
                    let mut deserializer = cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(
                        data,
                        cdr::Infinite,
                    );
                    serde::Deserialize::deserialize(&mut deserializer)
                        .map_err(|err| PreconditionNotMet(err.to_string()))
                }
                PL_CDR_BE => {
                    let mut deserializer =
                        ParameterListDeserializer::<byteorder::BigEndian>::new(data);
                    serde::Deserialize::deserialize(&mut deserializer)
                        .map_err(|err| PreconditionNotMet(err.to_string()))
                }
                PL_CDR_LE => {
                    let mut deserializer =
                        ParameterListDeserializer::<byteorder::LittleEndian>::new(data);
                    serde::Deserialize::deserialize(&mut deserializer)
                        .map_err(|err| PreconditionNotMet(err.to_string()))
                }
                XML => {
                    let mut deserializer = serde_xml_rs::Deserializer::new_from_reader(data);
                    serde::Deserialize::deserialize(&mut deserializer)
                        .map_err(|err| PreconditionNotMet(err.to_string()))
                }
                _ => Err(PreconditionNotMet(
                    "Illegal representation identifier".to_string(),
                )),
            }
        }
    }
}

pub fn dds_serialize_key<T>(value: &T) -> DdsResult<DdsSerializedKey>
where
    T: DdsGetKey,
{
    let mut writer = vec![];
    let mut serializer = cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut writer);
    let key = value.get_key();
    serde::Serialize::serialize(&key, &mut serializer)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;
    Ok(writer.into())
}

pub fn dds_serialize_key_to_bytes<T>(value: &T) -> DdsResult<DdsSerializedKey>
where
    T: DdsGetKey,
{
    let mut writer = vec![];
    writer
        .write_all(&CDR_LE)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;
    writer
        .write_all(&REPRESENTATION_OPTIONS)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;
    let mut serializer = cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(&mut writer);
    let key = value.get_key();
    serde::Serialize::serialize(&key, &mut serializer)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;
    Ok(writer.into())
}

pub fn dds_deserialize_key_from_bytes<T>(mut data: &[u8]) -> DdsResult<T::OwningKeyHolder>
where
    T: DdsSetKeyFields,
{
    let mut REPRESENTATION = [0u8, 0];
    data.read_exact(&mut REPRESENTATION)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;

    let mut representation_option = [0u8, 0];
    data.read_exact(&mut representation_option)
        .map_err(|err| PreconditionNotMet(err.to_string()))?;
    match REPRESENTATION {
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
        _ => Err(PreconditionNotMet(
            "Illegal representation identifier".to_string(),
        )),
    }
}

pub fn dds_set_key_fields_from_serialized_key<T>(
    value: &mut T,
    serialized_key: &[u8],
) -> DdsResult<()>
where
    T: DdsSetKeyFields,
{
    let key_holder = dds_deserialize_key_from_bytes::<T>(serialized_key)?;
    value.set_key_from_holder(key_holder);
    Ok(())
}
