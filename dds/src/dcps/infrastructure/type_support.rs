use crate::{
    infrastructure::error::DdsResult,
    xtypes::{
        dynamic_type::{DynamicData, DynamicType},
        xcdr_deserializer::{Xcdr2BeDeserializer, Xcdr2LeDeserializer},
    },
};
pub use dust_dds_derive::{DdsDeserialize, TypeSupport};

/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport {
    /// This operation returns the default name for the data-type represented by the TypeSupport.
    fn get_type_name() -> &'static str {
        core::any::type_name::<Self>()
    }

    /// This operation returns a ['DynamicType'] object corresponding to the TypeSupport’s data type
    fn get_type() -> DynamicType;

    /// Create a 'DynamicData' object with the contents of an input sample of the TypeSupport’s data type.
    fn create_dynamic_sample(self) -> DynamicData;
}

/// This trait describes how the bytes can be deserialize to construct the data structure.
///
/// This trait is typically used when reading the data from the samples from the DataReader.
/// The `'de` lifetime of this trait is the lifetime of data that may be borrowed from the input when deserializing.
///
/// ## Derivable
///
/// This trait can be automatically derived if the struct implements `XTypesDeserialize`.
pub trait DdsDeserialize<'de>: Sized {
    /// Method to deserialize the bytes into an instance of the type.
    fn deserialize_data(serialized_data: &'de [u8]) -> DdsResult<Self>;
}

use crate::xtypes::{
    deserialize::XTypesDeserialize,
    error::XTypesError,
    xcdr_deserializer::{Xcdr1BeDeserializer, Xcdr1LeDeserializer},
};
/// This is a convenience derive to allow the user to easily derive all the different traits needed for a type to be used for
/// communication with Dust DDS. If the individual traits are manually derived then this derive should not be used.
///
/// This trait can be automatically derived. The generated trait uses by default a CdrLe
/// representation and it determines whether the type is keyed or not depending on whether
/// any field is marked `#[dust_dds(key)]` or not.
///
/// An example of a typical usage of derive is the following:
///
/// ```rust
///     use dust_dds::infrastructure::type_support::DdsType;
///
///     #[derive(DdsType)]
///     struct KeyedData {
///         #[dust_dds(key)]
///         id: u8,
///         value: u32,
///     }
/// ```
///
/// It is also possible to derive structs with a lifetime:
///
/// ```rust
///     use dust_dds::infrastructure::type_support::DdsType;
///
///     #[derive(DdsType)]
///     struct BorrowedData<'a> {
///         #[dust_dds(key)]
///         id: u8,
///         value: &'a [u8],
///     }
/// ```
///
pub use dust_dds_derive::DdsType;

type RepresentationIdentifier = [u8; 2];
// type RepresentationOptions = [u8; 2];

const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const CDR2_BE: RepresentationIdentifier = [0x00, 0x06];
const CDR2_LE: RepresentationIdentifier = [0x00, 0x07];
const D_CDR2_BE: RepresentationIdentifier = [0x00, 0x08];
const D_CDR2_LE: RepresentationIdentifier = [0x00, 0x09];
const _PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const _PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
// const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];

// /// This is a helper function to serialize a type implementing [`XTypesSerialize`] using the XTypes defined XCDR1 representation with LittleEndian endianness.
// pub fn serialize_rtps_xtypes_xcdr1_le(value: &impl XTypesSerialize) -> DdsResult<Vec<u8>> {
//     let padded_length = (Xcdr1LeSerializer::bytes_len(value)? + 3) & !3;
//     let mut writer = Vec::with_capacity(padded_length + 4);
//     writer.extend_from_slice(&CDR_LE);
//     writer.extend_from_slice(&REPRESENTATION_OPTIONS);
//     let mut serializer = Xcdr1LeSerializer::new(&mut writer);
//     XTypesSerialize::serialize(value, &mut serializer)?;
//     pad(&mut writer);
//     Ok(writer)
// }

// /// This is a helper function to serialize a type implementing [`XTypesSerialize`] using the XTypes defined XCDR1 representation with BigEndian endianness.
// pub fn serialize_rtps_xtypes_xcdr1_be(value: &impl XTypesSerialize) -> DdsResult<Vec<u8>> {
//     let padded_length = (Xcdr1BeSerializer::bytes_len(value)? + 3) & !3;
//     let mut writer = Vec::with_capacity(padded_length + 4);
//     writer.extend_from_slice(&CDR_BE);
//     writer.extend_from_slice(&REPRESENTATION_OPTIONS);
//     let mut serializer = Xcdr1BeSerializer::new(&mut writer);
//     XTypesSerialize::serialize(value, &mut serializer)?;
//     pad(&mut writer);
//     Ok(writer)
// }

// fn pad(writer: &mut Vec<u8>) {
//     let padding = match writer.len() % 4 {
//         1 => &[0, 0, 0][..],
//         2 => &[0, 0][..],
//         3 => &[0][..],
//         _ => &[][..],
//     };
//     writer.extend_from_slice(padding);
//     writer[3] = padding.len() as u8;
// }

/// This is a helper function to deserialize a type implementing [`CdrDeserialize`] using the RTPS classic CDR representation.
/// The representation endianness to be used is automatically determined from the representation identifier and options
pub fn deserialize_rtps_encapsulated_data<'de, T>(serialized_data: &mut &'de [u8]) -> DdsResult<T>
where
    T: XTypesDeserialize<'de>,
{
    if serialized_data.len() < 4 {
        Err(XTypesError::InvalidData)?;
    }
    let representation_identifier = [serialized_data[0], serialized_data[1]];
    let _representation_option = [serialized_data[2], serialized_data[3]];
    *serialized_data = &serialized_data[4..];

    let value = match representation_identifier {
        CDR_BE => XTypesDeserialize::deserialize(&mut Xcdr1BeDeserializer::new(serialized_data)),
        CDR_LE => XTypesDeserialize::deserialize(&mut Xcdr1LeDeserializer::new(serialized_data)),
        CDR2_BE | D_CDR2_BE => {
            XTypesDeserialize::deserialize(&mut Xcdr2BeDeserializer::new(serialized_data))
        }
        CDR2_LE | D_CDR2_LE => {
            XTypesDeserialize::deserialize(&mut Xcdr2LeDeserializer::new(serialized_data))
        }
        _ => Err(XTypesError::InvalidData),
    }?;
    Ok(value)
}

// impl<T: Into<DataKind>> TypeSupport for Option<T> {
//     fn get_type() -> DynamicType {
//         todo!()
//     }

//     fn create_dynamic_sample(self) -> DynamicData {
//         todo!()
//     }
// }
