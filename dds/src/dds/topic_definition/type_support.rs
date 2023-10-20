use std::io::{Read, Write};

use crate::{
    implementation::parameter_list_serde::{
        serde_parameter_list_deserializer::ParameterListDeserializer,
        serde_parameter_list_serializer::ParameterListSerializer,
    },
    infrastructure::error::{DdsError::PreconditionNotMet, DdsResult},
};

pub use dust_dds_derive::{DdsBorrowKeyHolder, DdsHasKey, DdsRepresentation, DdsOwningKeyHolder, DdsType};

pub trait DdsSerialize {
    fn serialize_data(&self, writer: impl std::io::Write) -> DdsResult<()>;
}

pub trait DdsDeserialize<'de>: Sized {
    fn deserialize_data(serialized_data: &mut &'de [u8]) -> DdsResult<Self>;
}

pub trait DdsSerializeKeyFields {
    fn serialize_key_fields(&self, writer: impl std::io::Write) -> DdsResult<()>;
}

pub trait DdsGetKeyFromFoo {
    fn get_key_from(&self) -> DdsResult<DdsSerializedKey>;
}

pub trait DdsGetKeyFromSerializedData {
    fn get_key_from_serialized_data(serialized_data: &[u8]) -> DdsResult<DdsSerializedKey>;
}

pub trait DdsGetKeyFromSerializedKeyFields {
    fn get_key_from_serialized_key_fields(
        serialized_key_fields: &[u8],
    ) -> DdsResult<DdsSerializedKey>;
}

pub trait DdsHasKey {
    const HAS_KEY: bool;
}

pub enum Representation {
    CdrLe,
    CdrBe,
    PlCdrBe,
    PlCdrLe,
}

type RepresentationIdentifier = [u8; 2];
type RepresentationOptions = [u8; 2];

const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
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

pub trait DdsType: DdsRepresentation + DdsHasKey + DdsBorrowKeyHolder + DdsOwningKeyHolder {}

pub trait DdsRepresentation {
    const REPRESENTATION: Representation;
}

impl<T> DdsSerialize for T
where
    T: serde::Serialize + DdsRepresentation,
{
    fn serialize_data(&self, mut writer: impl std::io::Write) -> DdsResult<()> {
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
                serde::Serialize::serialize(self, &mut serializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))?;
            }
            Representation::CdrBe => {
                writer
                    .write_all(&CDR_BE)
                    .map_err(|err| PreconditionNotMet(err.to_string()))?;
                writer
                    .write_all(&REPRESENTATION_OPTIONS)
                    .map_err(|err| PreconditionNotMet(err.to_string()))?;
                let mut serializer =
                    cdr::ser::Serializer::<_, byteorder::BigEndian>::new(&mut writer);
                serde::Serialize::serialize(self, &mut serializer)
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
                serde::Serialize::serialize(self, &mut serializer)
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
                serde::Serialize::serialize(self, &mut serializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))?;
            }
        };
        Ok(())
    }
}

impl<'de, T> DdsDeserialize<'de> for T
where
    T: DdsRepresentation + serde::Deserialize<'de>,
{
    fn deserialize_data(serialized_data: &mut &'de [u8]) -> DdsResult<Self> {
        let mut representation_identifier = [0u8, 0];
        serialized_data
            .read_exact(&mut representation_identifier)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;

        let mut representation_option = [0u8, 0];
        serialized_data
            .read_exact(&mut representation_option)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;

        match representation_identifier {
            CDR_BE => {
                let mut deserializer = cdr::Deserializer::<_, _, byteorder::BigEndian>::new(
                    serialized_data,
                    cdr::Infinite,
                );
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            CDR_LE => {
                let mut deserializer = cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(
                    serialized_data,
                    cdr::Infinite,
                );
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            PL_CDR_BE => {
                let mut deserializer =
                    ParameterListDeserializer::<byteorder::BigEndian>::new(serialized_data);
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            PL_CDR_LE => {
                let mut deserializer =
                    ParameterListDeserializer::<byteorder::LittleEndian>::new(serialized_data);
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            _ => Err(PreconditionNotMet(
                "Illegal representation identifier".to_string(),
            )),
        }
    }
}

pub trait DdsBorrowKeyHolder {
    // Serde trait bounds placed here since there is no easy way to express this bound
    // without specifying the lifetime 'a which makes the bounds on usage very complicated
    type BorrowedKeyHolder<'a>: serde::Serialize
    where
        Self: 'a;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_>;
}

impl<T> DdsSerializeKeyFields for T
where
    T: DdsBorrowKeyHolder,
    for<'a> T::BorrowedKeyHolder<'a>: serde::Serialize,
{
    fn serialize_key_fields(&self, mut writer: impl std::io::Write) -> DdsResult<()> {
        writer
            .write_all(&CDR_LE)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;
        writer
            .write_all(&REPRESENTATION_OPTIONS)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;
        let mut serializer = cdr::ser::Serializer::<_, byteorder::LittleEndian>::new(writer);
        let key = self.get_key();
        serde::Serialize::serialize(&key, &mut serializer)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;
        Ok(())
    }
}

impl<T> DdsGetKeyFromFoo for T
where
    T: DdsBorrowKeyHolder,
    for<'a> T::BorrowedKeyHolder<'a>: serde::Serialize,
{
    fn get_key_from(&self) -> DdsResult<DdsSerializedKey> {
        let mut writer = Vec::new();
        let mut serializer = cdr::ser::Serializer::<_, byteorder::BigEndian>::new(&mut writer);
        let key = self.get_key();
        serde::Serialize::serialize(&key, &mut serializer)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;
        Ok(writer.into())
    }
}

pub trait DdsOwningKeyHolder {
    type OwningKeyHolder;
}

impl<T> DdsGetKeyFromSerializedData for T
where
    T: DdsOwningKeyHolder + DdsRepresentation,
    T::OwningKeyHolder: for<'de> serde::Deserialize<'de> + serde::Serialize,
{
    fn get_key_from_serialized_data(mut serialized_data: &[u8]) -> DdsResult<DdsSerializedKey> {
        let serialized_data = &mut serialized_data;
        let mut representation_identifier = [0u8, 0];
        serialized_data
            .read_exact(&mut representation_identifier)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;

        let mut representation_option = [0u8, 0];
        serialized_data
            .read_exact(&mut representation_option)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;

        let key_holder: T::OwningKeyHolder = match representation_identifier {
            CDR_BE => {
                let mut deserializer = cdr::Deserializer::<_, _, byteorder::BigEndian>::new(
                    serialized_data,
                    cdr::Infinite,
                );
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            CDR_LE => {
                let mut deserializer = cdr::Deserializer::<_, _, byteorder::LittleEndian>::new(
                    serialized_data,
                    cdr::Infinite,
                );
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            PL_CDR_BE => {
                let mut deserializer =
                    ParameterListDeserializer::<byteorder::BigEndian>::new(serialized_data);
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            PL_CDR_LE => {
                let mut deserializer =
                    ParameterListDeserializer::<byteorder::LittleEndian>::new(serialized_data);
                serde::Deserialize::deserialize(&mut deserializer)
                    .map_err(|err| PreconditionNotMet(err.to_string()))
            }
            _ => Err(PreconditionNotMet(
                "Illegal representation identifier".to_string(),
            )),
        }?;

        let mut writer = Vec::new();
        let mut serializer = cdr::ser::Serializer::<_, byteorder::BigEndian>::new(&mut writer);
        serde::Serialize::serialize(&key_holder, &mut serializer)
            .map_err(|err| PreconditionNotMet(err.to_string()))?;
        Ok(writer.into())
    }
}

macro_rules! implement_dds_get_key_for_built_in_type {
    ($t:ty) => {
        impl DdsBorrowKeyHolder for $t {
            type BorrowedKeyHolder<'a> = $t;

            fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
                *self
            }
        }

        impl DdsOwningKeyHolder for $t {
            type OwningKeyHolder = $t;
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

impl<T> DdsBorrowKeyHolder for Vec<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type BorrowedKeyHolder<'a> = &'a Vec<T> where T:'a;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self
    }
}

impl<T> DdsOwningKeyHolder for Vec<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type OwningKeyHolder = Vec<T>;
}

impl DdsBorrowKeyHolder for String {
    type BorrowedKeyHolder<'a> = &'a str;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self
    }
}

impl DdsOwningKeyHolder for String {
    type OwningKeyHolder = String;
}

impl<const N: usize, T> DdsBorrowKeyHolder for [T; N]
where
    [T; N]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type BorrowedKeyHolder<'a> = &'a [T; N] where T:'a;

    fn get_key(&self) -> Self::BorrowedKeyHolder<'_> {
        self
    }
}

impl<const N: usize, T> DdsOwningKeyHolder for [T; N]
where
    [T; N]: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type OwningKeyHolder = [T; N];
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
    };
    Ok(writer)
}

pub fn dds_deserialize_from_bytes<'de, T>(data: &mut &'de [u8]) -> DdsResult<T>
where
    T: serde::Deserialize<'de> + DdsRepresentation,
{
    match T::REPRESENTATION {
        _ => {
            let mut representation_identifier = [0u8, 0];
            data.read_exact(&mut representation_identifier)
                .map_err(|err| PreconditionNotMet(err.to_string()))?;

            let mut representation_option = [0u8, 0];
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
                _ => Err(PreconditionNotMet(
                    "Illegal representation identifier".to_string(),
                )),
            }
        }
    }
}

pub fn dds_serialize_key<T>(value: &T) -> DdsResult<DdsSerializedKey>
where
    T: DdsBorrowKeyHolder,
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
    T: DdsBorrowKeyHolder,
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
