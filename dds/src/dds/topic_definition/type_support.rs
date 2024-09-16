use crate::{
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
    },
    xtypes::{
        dynamic_type::{
            DynamicType, DynamicTypeMember, ExtensibilityKind, MemberDescriptor, MemberId,
            ObjectName, TryConstructKind, TypeDescriptor,
        },
        type_object::{
            CompleteAnnotationParameter, CompleteBitfield, CompleteBitflag, CompleteBitmaskType,
            CompleteEnumeratedLiteral, CompleteStructMember, CompleteTypeObject,
            CompleteUnionMember, MinimalTypeObject, TypeIdentifier, TK_ALIAS, TK_ANNOTATION,
            TK_ARRAY, TK_BITMASK, TK_BITSET, TK_BOOLEAN, TK_BYTE, TK_CHAR16, TK_CHAR8, TK_ENUM,
            TK_FLOAT128, TK_FLOAT32, TK_FLOAT64, TK_INT16, TK_INT32, TK_INT64, TK_INT8, TK_MAP,
            TK_NONE, TK_SEQUENCE, TK_STRING16, TK_STRING8, TK_STRUCTURE, TK_UINT16, TK_UINT32,
            TK_UINT64, TK_UINT8, TK_UNION,
        },
    },
};
pub use dust_dds_derive::{DdsDeserialize, DdsHasKey, DdsSerialize, DdsTypeXml};
use std::io::{Read, Write};

/// The TypeSupport trait represents a type that can be transmitted by DDS.
pub trait TypeSupport {
    /// This operation returns the default name for the data-type represented by the TypeSupport.
    fn get_type_name() -> &'static str;

    /// This operation returns a ['DynamicType'] object corresponding to the TypeSupport’s data type
    fn get_type() -> impl DynamicType;
}

impl DynamicType for TypeIdentifier {
    fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => Ok(TypeDescriptor {
                kind: self.get_kind(),
                name: self.get_name(),
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            }),
            TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
            TypeIdentifier::EkComplete { complete } => complete.get_descriptor(),
            TypeIdentifier::EkMinimal { minimal } => minimal.get_descriptor(),
        }
    }

    fn get_name(&self) -> ObjectName {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => String::new(),
            TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
            TypeIdentifier::EkComplete { complete } => complete.get_name(),
            TypeIdentifier::EkMinimal { minimal } => minimal.get_name(),
        }
    }

    fn get_kind(&self) -> crate::xtypes::type_object::TypeKind {
        match self {
            TypeIdentifier::TkNone => TK_NONE,
            TypeIdentifier::TkBoolean => TK_BOOLEAN,
            TypeIdentifier::TkByteType => TK_BYTE,
            TypeIdentifier::TkInt8Type => TK_INT8,
            TypeIdentifier::TkInt16Type => TK_INT16,
            TypeIdentifier::TkInt32Type => TK_INT32,
            TypeIdentifier::TkInt64Type => TK_INT64,
            TypeIdentifier::TkUint8Type => TK_UINT8,
            TypeIdentifier::TkUint16Type => TK_UINT16,
            TypeIdentifier::TkUint32Type => TK_UINT32,
            TypeIdentifier::TkUint64Type => TK_UINT64,
            TypeIdentifier::TkFloat32Type => TK_FLOAT32,
            TypeIdentifier::TkFloat64Type => TK_FLOAT64,
            TypeIdentifier::TkFloat128Type => TK_FLOAT128,
            TypeIdentifier::TkChar8Type => TK_CHAR8,
            TypeIdentifier::TkChar16Type => TK_CHAR16,
            TypeIdentifier::TiString8Small { .. } | TypeIdentifier::TiString8Large { .. } => {
                TK_STRING8
            }
            TypeIdentifier::TiString16Small { .. } | TypeIdentifier::TiString16Large { .. } => {
                TK_STRING16
            }
            TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. } => TK_SEQUENCE,
            TypeIdentifier::TiPlainArraySmall { .. } | TypeIdentifier::TiPlainArrayLarge { .. } => {
                TK_ARRAY
            }
            TypeIdentifier::TiPlainMapSmall { .. } | TypeIdentifier::TiPlainMapLarge { .. } => {
                TK_MAP
            }
            TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
            TypeIdentifier::EkComplete { complete } => complete.get_kind(),
            TypeIdentifier::EkMinimal { minimal } => minimal.get_kind(),
        }
    }

    fn get_member_count(&self) -> u32 {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => 0,
            TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
            TypeIdentifier::EkComplete { complete } => complete.get_member_count(),
            TypeIdentifier::EkMinimal { minimal } => minimal.get_member_count(),
        }
    }

    fn get_member_by_index(&self, index: u32) -> Result<&dyn DynamicTypeMember, XTypesError> {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => Err(XTypesError::InvalidIndex),
            TypeIdentifier::TiStronglyConnectedComponent { .. } => todo!(),
            TypeIdentifier::EkComplete { complete } => complete.get_member_by_index(index),
            TypeIdentifier::EkMinimal { minimal } => minimal.get_member_by_index(index),
        }
    }
}

impl DynamicType for CompleteTypeObject {
    fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        match self {
            CompleteTypeObject::TkAlias { alias_type } => {
                alias_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkAnnotation { annotation_type } => {
                annotation_type.header.annotation_name.clone()
            }
            CompleteTypeObject::TkStructure { struct_type } => {
                struct_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkUnion { union_type } => {
                union_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkBitset { bitset_type } => {
                bitset_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkSequence { .. }
            | CompleteTypeObject::TkArray { .. }
            | CompleteTypeObject::TkMap { .. } => String::new(),
            CompleteTypeObject::TkEnum { enumerated_type } => {
                enumerated_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkBitmask { bitmask_type } => {
                bitmask_type.header.detail.type_name.clone()
            }
        }
    }

    fn get_kind(&self) -> crate::xtypes::type_object::TypeKind {
        match self {
            CompleteTypeObject::TkAlias { .. } => TK_ALIAS,
            CompleteTypeObject::TkAnnotation { .. } => TK_ANNOTATION,
            CompleteTypeObject::TkStructure { .. } => TK_STRUCTURE,
            CompleteTypeObject::TkUnion { .. } => TK_UNION,
            CompleteTypeObject::TkBitset { .. } => TK_BITSET,
            CompleteTypeObject::TkSequence { .. } => TK_SEQUENCE,
            CompleteTypeObject::TkArray { .. } => TK_ARRAY,
            CompleteTypeObject::TkMap { .. } => TK_MAP,
            CompleteTypeObject::TkEnum { .. } => TK_ENUM,
            CompleteTypeObject::TkBitmask { .. } => TK_BITMASK,
        }
    }

    fn get_member_count(&self) -> u32 {
        match self {
            CompleteTypeObject::TkAlias { alias_type } => todo!(),
            CompleteTypeObject::TkAnnotation { annotation_type } => todo!(),
            CompleteTypeObject::TkStructure { struct_type } => struct_type.member_seq.len() as u32,
            CompleteTypeObject::TkUnion { union_type } => union_type.member_seq.len() as u32,
            CompleteTypeObject::TkBitset { bitset_type } => todo!(),
            CompleteTypeObject::TkSequence { .. }
            | CompleteTypeObject::TkArray { .. }
            | CompleteTypeObject::TkMap { .. } => 0,
            CompleteTypeObject::TkEnum { enumerated_type } => {
                enumerated_type.literal_seq.len() as u32
            }
            CompleteTypeObject::TkBitmask { bitmask_type } => todo!(),
        }
    }

    fn get_member_by_index(
        &self,
        index: u32,
    ) -> Result<&dyn crate::xtypes::dynamic_type::DynamicTypeMember, XTypesError> {
        match self {
            CompleteTypeObject::TkAlias { .. } => Err(XTypesError::InvalidIndex),
            CompleteTypeObject::TkAnnotation { annotation_type } => Ok(annotation_type
                .member_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkStructure { struct_type } => Ok(struct_type
                .member_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkUnion { union_type } => Ok(union_type
                .member_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkBitset { bitset_type } => Ok(bitset_type
                .field_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkSequence { .. }
            | CompleteTypeObject::TkArray { .. }
            | CompleteTypeObject::TkMap { .. } => Err(XTypesError::InvalidIndex),
            CompleteTypeObject::TkEnum { enumerated_type } => Ok(enumerated_type
                .literal_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkBitmask { bitmask_type } => Ok(bitmask_type
                .flag_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
        }
    }
}

impl DynamicType for MinimalTypeObject {
    fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        todo!()
    }

    fn get_kind(&self) -> crate::xtypes::type_object::TypeKind {
        todo!()
    }

    fn get_member_count(&self) -> u32 {
        todo!()
    }

    fn get_member_by_index(&self, index: u32) -> Result<&dyn DynamicTypeMember, XTypesError> {
        todo!()
    }
}

impl DynamicTypeMember for CompleteStructMember {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            _type: &self.common.member_type_id,
            default_value: "",
            index: 0,
            try_construct_kind: TryConstructKind::Discard,
            is_key: self.common.member_flags.is_key,
            is_optional: self.common.member_flags.is_optional,
            is_must_understand: self.common.member_flags.is_must_undestand,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        self.common.member_id
    }

    fn get_name(&self) -> ObjectName {
        self.detail.name.clone()
    }
}

impl DynamicTypeMember for CompleteEnumeratedLiteral {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            _type: &TypeIdentifier::TkNone,
            default_value: "",
            index: 0,
            try_construct_kind: TryConstructKind::Discard,
            is_key: false,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        todo!()
    }
}

impl DynamicTypeMember for CompleteUnionMember {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        todo!()
    }

    fn get_id(&self) -> MemberId {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        todo!()
    }
}

impl DynamicTypeMember for CompleteAnnotationParameter {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        todo!()
    }

    fn get_id(&self) -> MemberId {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        todo!()
    }
}

impl DynamicTypeMember for CompleteBitfield {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        todo!()
    }

    fn get_id(&self) -> MemberId {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        todo!()
    }
}

impl DynamicTypeMember for CompleteBitflag {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        todo!()
    }

    fn get_id(&self) -> MemberId {
        todo!()
    }

    fn get_name(&self) -> ObjectName {
        todo!()
    }
}

#[doc(hidden)]
pub trait DynamicTypeInterface {
    fn has_key(&self) -> bool;

    fn get_serialized_key_from_serialized_foo(&self, serialized_foo: &[u8]) -> DdsResult<Vec<u8>>;

    fn instance_handle_from_serialized_foo(
        &self,
        serialized_foo: &[u8],
    ) -> DdsResult<InstanceHandle>;

    fn instance_handle_from_serialized_key(
        &self,
        serialized_key: &[u8],
    ) -> DdsResult<InstanceHandle>;

    fn xml_type(&self) -> String;

    fn user_data(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

/// This trait indicates whether the associated type is keyed or not, i.e. if the middleware
/// should manage different instances of the type.
///
/// ## Derivable
///
/// This trait can be automatically derived. If the struct has any field marked `#[dust_dds(key)]`
/// then HAS_KEY will be set to 'true' otherwise will be set to 'false'.
pub trait DdsHasKey {
    /// Constant indicating whether the associated type has a key or not.
    const HAS_KEY: bool;
}

/// This trait defines how to serialize the information contained in a data structure to be published.
///
/// The information generated by the method of this trait is typically visible on the
/// `serializedData` element of the Data submessage when transmitting a published sample.
///
/// ## Derivable
///
/// This trait can be automatically derived if the struct implements `XTypesSerialize`.
pub trait DdsSerialize {
    /// Method to serialize the instance of the type into the provided writer.
    fn serialize_data(&self) -> DdsResult<Vec<u8>>;
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

/// This trait defines the key associated with the type. The key is used to identify different instances of the type.
/// The returned key object must implement [`XTypesSerialize`] and [`XTypesDeserialize`] since CDR is the format always
/// used to transmit the key information on the wire and this can not be modified by the user.
///
/// ## Derivable
///
/// This trait can be automatically derived if all the field marked `#[dust_dds(key)]` implement [`XTypesSerialize`] and [`XTypesDeserialize`]
///
pub trait DdsKey {
    /// Type representing the key for the type in which this trait is implemented
    type Key: XTypesSerialize + for<'de> XTypesDeserialize<'de>;

    /// Method to get the key from an instance of the type.
    fn get_key(&self) -> DdsResult<Self::Key>;

    /// Method to get the key from a serialized instance of the type.
    fn get_key_from_serialized_data(serialized_foo: &[u8]) -> DdsResult<Self::Key>;
}

/// This trait defines the optional type representation for a user type. The type representation
/// returned by the function in this trait must follow the description in 7.3.2 XML Type Representation
/// of the [OMG DDS-XTypes standard](https://www.omg.org/spec/DDS-XTypes/1.3/).
///
/// ## Derivable
///
/// This trait can be automatically derived for every DustDDS supported type.
pub trait DdsTypeXml {
    /// Method to get the XML representation of a type.
    fn get_type_xml() -> Option<String>;
}

use crate::xtypes::{
    deserialize::XTypesDeserialize,
    error::XTypesError,
    serialize::XTypesSerialize,
    xcdr_deserializer::{Xcdr1BeDeserializer, Xcdr1LeDeserializer},
    xcdr_serializer::{Xcdr1BeSerializer, Xcdr1LeSerializer},
};
/// This is a convenience derive to allow the user to easily derive all the different traits needed for a type to be used for
/// communication with DustDDS. If the individual traits are manually derived then this derive should not be used.
///
/// This trait can be automatically derived. The generated trait uses by default a CdrLe
/// representation and it determines whether the type is keyed or not depending on whether
/// any field is marked `#[dust_dds(key)]` or not.
///
/// An example of a typical usage of derive is the following:
///
/// ```rust
///     use dust_dds::topic_definition::type_support::DdsType;
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
///     use dust_dds::topic_definition::type_support::DdsType;
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
type RepresentationOptions = [u8; 2];

const CDR_BE: RepresentationIdentifier = [0x00, 0x00];
const CDR_LE: RepresentationIdentifier = [0x00, 0x01];
const _CDR2_BE: RepresentationIdentifier = [0x00, 0x06];
const _CDR2_LE: RepresentationIdentifier = [0x00, 0x07];
const _D_CDR2_BE: RepresentationIdentifier = [0x00, 0x08];
const _D_CDR2_LE: RepresentationIdentifier = [0x00, 0x09];
const _PL_CDR_BE: RepresentationIdentifier = [0x00, 0x02];
const _PL_CDR_LE: RepresentationIdentifier = [0x00, 0x03];
const REPRESENTATION_OPTIONS: RepresentationOptions = [0x00, 0x00];

/// This is a helper function to serialize a type implementing [`XTypesSerialize`] using the XTypes defined XCDR1 representation with LittleEndian endianness.
pub fn serialize_rtps_xtypes_xcdr1_le(value: &impl XTypesSerialize) -> DdsResult<Vec<u8>> {
    let padded_length = (Xcdr1LeSerializer::bytes_len(value)? + 3) & !3;
    let mut writer = Vec::with_capacity(padded_length + 4);
    writer.write_all(&CDR_LE)?;
    writer.write_all(&REPRESENTATION_OPTIONS)?;
    let mut serializer = Xcdr1LeSerializer::new(&mut writer);
    XTypesSerialize::serialize(value, &mut serializer)?;
    pad(&mut writer)?;
    Ok(writer)
}

/// This is a helper function to serialize a type implementing [`XTypesSerialize`] using the XTypes defined XCDR1 representation with BigEndian endianness.
pub fn serialize_rtps_xtypes_xcdr1_be(value: &impl XTypesSerialize) -> DdsResult<Vec<u8>> {
    let padded_length = (Xcdr1BeSerializer::bytes_len(value)? + 3) & !3;
    let mut writer = Vec::with_capacity(padded_length + 4);
    writer.write_all(&CDR_LE)?;
    writer.write_all(&REPRESENTATION_OPTIONS)?;
    let mut serializer = Xcdr1BeSerializer::new(&mut writer);
    XTypesSerialize::serialize(value, &mut serializer)?;
    pad(&mut writer)?;
    Ok(writer)
}

fn pad(writer: &mut Vec<u8>) -> std::io::Result<()> {
    let padding = match writer.len() % 4 {
        1 => &[0, 0, 0][..],
        2 => &[0, 0][..],
        3 => &[0][..],
        _ => &[][..],
    };
    writer.write_all(padding)?;
    writer[3] = padding.len() as u8;
    Ok(())
}

/// This is a helper function to deserialize a type implementing [`CdrDeserialize`] using the RTPS classic CDR representation.
/// The representation endianness to be used is automatically determined from the representation identifier and options
pub fn deserialize_rtps_encapsulated_data<'de, T>(serialized_data: &mut &'de [u8]) -> DdsResult<T>
where
    T: XTypesDeserialize<'de>,
{
    let mut representation_identifier = [0u8, 0];
    serialized_data
        .read_exact(&mut representation_identifier)
        .map_err(|err| DdsError::Error(err.to_string()))?;

    let mut representation_option = [0u8, 0];
    serialized_data
        .read_exact(&mut representation_option)
        .map_err(|err| DdsError::Error(err.to_string()))?;

    let value = match representation_identifier {
        CDR_BE => XTypesDeserialize::deserialize(&mut Xcdr1BeDeserializer::new(serialized_data)),
        CDR_LE => XTypesDeserialize::deserialize(&mut Xcdr1LeDeserializer::new(serialized_data)),
        _ => Err(XTypesError::InvalidData),
    }?;
    Ok(value)
}
