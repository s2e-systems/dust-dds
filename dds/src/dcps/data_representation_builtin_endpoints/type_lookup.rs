//! TypeLookup Service types for XTypes 1.3 Type Discovery
//!
//! This module defines the DDS-RPC request/reply structures for the TypeLookup service,
//! which enables runtime type discovery between DDS participants.
//!
//! The TypeLookup service uses pre-defined entity IDs (XTypes 1.3 Table 61):
//! - Request writer/reader: [0x00, 0x03, 0x00]
//! - Reply writer/reader: [0x00, 0x03, 0x01]

use crate::{
    infrastructure::type_support::TypeSupport,
    transport::types::{Guid, SequenceNumber},
    xtypes::{
        binding::XTypesBinding,
        data_storage::{DataStorage, DataStorageMapping},
        dynamic_type::{
            DynamicDataFactory, DynamicType, DynamicTypeBuilder, DynamicTypeBuilderFactory,
            ExtensibilityKind, MemberDescriptor, TryConstructKind, TypeDescriptor, TypeKind,
        },
        error::{XTypesError, XTypesResult},
    },
};
use alloc::{string::String, vec::Vec};

// ============================================================================
// DDS-RPC Types (from DDS-RPC spec)
// ============================================================================

/// SampleIdentity identifies a specific DDS sample.
/// Used in DDS-RPC to correlate requests with replies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleIdentity {
    /// GUID of the writer that produced the sample
    pub writer_guid: Guid,
    /// Sequence number assigned by the writer
    pub sequence_number: SequenceNumber,
}

impl Default for SampleIdentity {
    fn default() -> Self {
        Self {
            writer_guid: Guid::new([0; 12], crate::transport::types::ENTITYID_UNKNOWN),
            sequence_number: 0,
        }
    }
}

impl XTypesBinding for SampleIdentity {
    fn get_dynamic_type() -> DynamicType {
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: String::from("SampleIdentity"),
            base_type: None,
            discriminator_type: None,
            bound: alloc::vec::Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("writer_guid"),
                id: 0,
                r#type: Guid::get_dynamic_type(),
                default_value: None,
                index: 0,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("sequence_number"),
                id: 1,
                r#type: i64::get_dynamic_type(),
                default_value: None,
                index: 1,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder.build()
    }
}

impl DataStorageMapping for SampleIdentity {
    fn into_storage(self) -> DataStorage {
        let mut data = DynamicDataFactory::create_data(Self::get_dynamic_type());
        data.set_value(0, self.writer_guid.into_storage());
        data.set_value(1, self.sequence_number.into_storage());
        DataStorage::ComplexValue(data)
    }

    fn try_from_storage(storage: DataStorage) -> XTypesResult<Self> {
        match storage {
            DataStorage::ComplexValue(mut data) => Ok(Self {
                writer_guid: Guid::try_from_storage(
                    data.remove_value(0).map_err(|_| XTypesError::InvalidType)?,
                )?,
                sequence_number: i64::try_from_storage(
                    data.remove_value(1).map_err(|_| XTypesError::InvalidType)?,
                )?,
            }),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

/// Remote exception code returned in RPC replies.
/// 0 = success, non-zero indicates an exception.
pub type RemoteExceptionCode = u32;

/// Constants for RemoteExceptionCode
pub const REMOTE_EX_OK: RemoteExceptionCode = 0;
pub const REMOTE_EX_UNSUPPORTED: RemoteExceptionCode = 1;
pub const REMOTE_EX_INVALID_ARGUMENT: RemoteExceptionCode = 2;
pub const REMOTE_EX_OUT_OF_RESOURCES: RemoteExceptionCode = 3;
pub const REMOTE_EX_UNKNOWN_OPERATION: RemoteExceptionCode = 4;
pub const REMOTE_EX_UNKNOWN_EXCEPTION: RemoteExceptionCode = 5;

/// RequestHeader for DDS-RPC requests
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestHeader {
    /// Identifies this specific request
    pub request_id: SampleIdentity,
    /// Service instance name (typically empty for TypeLookup)
    pub instance_name: String,
}

impl Default for RequestHeader {
    fn default() -> Self {
        Self {
            request_id: SampleIdentity::default(),
            instance_name: String::new(),
        }
    }
}

impl XTypesBinding for RequestHeader {
    fn get_dynamic_type() -> DynamicType {
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: String::from("RequestHeader"),
            base_type: None,
            discriminator_type: None,
            bound: alloc::vec::Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("request_id"),
                id: 0,
                r#type: SampleIdentity::get_dynamic_type(),
                default_value: None,
                index: 0,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("instance_name"),
                id: 1,
                r#type: String::get_dynamic_type(),
                default_value: None,
                index: 1,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder.build()
    }
}

impl DataStorageMapping for RequestHeader {
    fn into_storage(self) -> DataStorage {
        let mut data = DynamicDataFactory::create_data(Self::get_dynamic_type());
        data.set_value(0, self.request_id.into_storage());
        data.set_value(1, self.instance_name.into_storage());
        DataStorage::ComplexValue(data)
    }

    fn try_from_storage(storage: DataStorage) -> XTypesResult<Self> {
        match storage {
            DataStorage::ComplexValue(mut data) => Ok(Self {
                request_id: SampleIdentity::try_from_storage(
                    data.remove_value(0).map_err(|_| XTypesError::InvalidType)?,
                )?,
                instance_name: String::try_from_storage(
                    data.remove_value(1).map_err(|_| XTypesError::InvalidType)?,
                )?,
            }),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

/// ReplyHeader for DDS-RPC replies
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyHeader {
    /// Identifies the request this reply corresponds to
    pub related_request_id: SampleIdentity,
    /// Exception code (0 = success)
    pub remote_ex: RemoteExceptionCode,
}

impl Default for ReplyHeader {
    fn default() -> Self {
        Self {
            related_request_id: SampleIdentity::default(),
            remote_ex: REMOTE_EX_OK,
        }
    }
}

impl XTypesBinding for ReplyHeader {
    fn get_dynamic_type() -> DynamicType {
        let mut builder = DynamicTypeBuilderFactory::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: String::from("ReplyHeader"),
            base_type: None,
            discriminator_type: None,
            bound: alloc::vec::Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind: ExtensibilityKind::Final,
            is_nested: true,
        });
        builder
            .add_member(MemberDescriptor {
                name: String::from("related_request_id"),
                id: 0,
                r#type: SampleIdentity::get_dynamic_type(),
                default_value: None,
                index: 0,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder
            .add_member(MemberDescriptor {
                name: String::from("remote_ex"),
                id: 1,
                r#type: u32::get_dynamic_type(),
                default_value: None,
                index: 1,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        builder.build()
    }
}

impl DataStorageMapping for ReplyHeader {
    fn into_storage(self) -> DataStorage {
        let mut data = DynamicDataFactory::create_data(Self::get_dynamic_type());
        data.set_value(0, self.related_request_id.into_storage());
        data.set_value(1, self.remote_ex.into_storage());
        DataStorage::ComplexValue(data)
    }

    fn try_from_storage(storage: DataStorage) -> XTypesResult<Self> {
        match storage {
            DataStorage::ComplexValue(mut data) => Ok(Self {
                related_request_id: SampleIdentity::try_from_storage(
                    data.remove_value(0).map_err(|_| XTypesError::InvalidType)?,
                )?,
                remote_ex: u32::try_from_storage(
                    data.remove_value(1).map_err(|_| XTypesError::InvalidType)?,
                )?,
            }),
            _ => Err(XTypesError::InvalidType),
        }
    }
}

// ============================================================================
// TypeLookup Operation Hash IDs (from XTypes 1.3 @hashid annotations)
// ============================================================================

/// Hash ID for getTypes operation
pub const TYPELOOKUP_GETTYPES_HASH: u32 = 0x018252d3;

/// Hash ID for getTypeDependencies operation
pub const TYPELOOKUP_GETDEPENDENCIES_HASH: u32 = 0x05aafb31;

// ============================================================================
// TypeLookup Service Types
// ============================================================================

/// Equivalence hash from TypeIdentifier - first 14 bytes of MD5 hash
pub type EquivalenceHash = [u8; 14];

/// Extract the equivalence hash from XCDR-encoded TypeIdentifier bytes.
///
/// TypeIdentifier for hashed types has format:
/// - 1 byte: discriminator (0xF1 = EK_MINIMAL, 0xF2 = EK_COMPLETE)
/// - 14 bytes: equivalence hash
///
/// Returns None if the TypeIdentifier is not a hashed type or malformed.
pub fn extract_hash_from_type_identifier(type_id_bytes: &[u8]) -> Option<EquivalenceHash> {
    if type_id_bytes.len() < 15 {
        return None;
    }
    let kind = type_id_bytes[0];
    // EK_MINIMAL = 0xF1, EK_COMPLETE = 0xF2
    if kind != 0xF1 && kind != 0xF2 {
        return None;
    }
    let mut hash = [0u8; 14];
    hash.copy_from_slice(&type_id_bytes[1..15]);
    Some(hash)
}

/// Compute the equivalence hash for a DynamicType.
///
/// The equivalence hash is the first 14 bytes of MD5 of a serialized
/// representation of the type. For simplicity, we use the type name
/// as the serialization input, which provides a stable hash for type
/// identification within a domain.
///
/// Note: For full XTypes interop, this should serialize the complete
/// TypeObject using XCDR2 LE. This simplified version works for
/// internal type registry operations.
pub fn compute_equivalence_hash(dynamic_type: &crate::xtypes::dynamic_type::DynamicType) -> EquivalenceHash {
    // Delegate to DynamicType's method which uses MD5(serialized TypeObject)
    // This ensures consistency with the hash in TypeInformation sent during discovery
    dynamic_type.compute_equivalence_hash()
}

/// Input for getTypes operation.
/// Contains TypeIdentifier hashes to look up.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeLookupGetTypesIn {
    /// List of type identifier hashes to look up (XCDR-encoded TypeIdentifiers)
    pub type_ids: Vec<Vec<u8>>,
}

/// A pair of TypeIdentifier (as hash) and TypeObject (as XCDR bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeIdentifierTypeObjectPairBytes {
    /// XCDR-encoded TypeIdentifier
    pub type_identifier: Vec<u8>,
    /// XCDR-encoded TypeObject
    pub type_object: Vec<u8>,
}

/// A pair of two TypeIdentifier hashes (complete -> minimal mapping)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeIdentifierPairBytes {
    /// First TypeIdentifier (typically complete)
    pub type_identifier1: Vec<u8>,
    /// Second TypeIdentifier (typically minimal)
    pub type_identifier2: Vec<u8>,
}

/// Output for getTypes operation
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeLookupGetTypesOut {
    /// TypeObject for each requested TypeIdentifier
    pub types: Vec<TypeIdentifierTypeObjectPairBytes>,
    /// Optional mappings from complete to minimal TypeIdentifiers
    pub complete_to_minimal: Vec<TypeIdentifierPairBytes>,
}

/// Input for getTypeDependencies operation
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeLookupGetTypeDependenciesIn {
    /// TypeIdentifiers to get dependencies for
    pub type_ids: Vec<Vec<u8>>,
    /// Continuation point for paginated results (empty for first request)
    pub continuation_point: Vec<u8>,
}

/// TypeIdentifier with serialized size info
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeIdentifierWithSizeBytes {
    /// XCDR-encoded TypeIdentifier
    pub type_id: Vec<u8>,
    /// Size of the serialized TypeObject
    pub typeobject_serialized_size: u32,
}

/// Output for getTypeDependencies operation
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeLookupGetTypeDependenciesOut {
    /// Dependent type identifiers with sizes
    pub dependent_typeids: Vec<TypeIdentifierWithSizeBytes>,
    /// Continuation point for next page (empty if complete)
    pub continuation_point: Vec<u8>,
}

/// TypeLookup_Call union - discriminated by operation hash
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeLookupCall {
    /// getTypes operation
    GetTypes(TypeLookupGetTypesIn),
    /// getTypeDependencies operation
    GetTypeDependencies(TypeLookupGetTypeDependenciesIn),
    /// Unknown operation
    Unknown(u32),
}

impl Default for TypeLookupCall {
    fn default() -> Self {
        TypeLookupCall::GetTypes(TypeLookupGetTypesIn::default())
    }
}

/// TypeLookup_Return union - discriminated by operation hash
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeLookupReturn {
    /// getTypes result
    GetTypes(TypeLookupGetTypesOut),
    /// getTypeDependencies result
    GetTypeDependencies(TypeLookupGetTypeDependenciesOut),
    /// Unknown operation result (raw bytes)
    Unknown(u32, Vec<u8>),
}

impl Default for TypeLookupReturn {
    fn default() -> Self {
        TypeLookupReturn::GetTypes(TypeLookupGetTypesOut::default())
    }
}

/// Complete TypeLookup request message
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeLookupRequest {
    /// DDS-RPC header
    pub header: RequestHeader,
    /// The operation and its parameters
    pub data: TypeLookupCall,
}

/// Complete TypeLookup reply message
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeLookupReply {
    /// DDS-RPC header
    pub header: ReplyHeader,
    /// The operation result
    pub data: TypeLookupReturn,
}

// ============================================================================
// TypeSupport implementations for TypeLookup request/reply
// ============================================================================

// Helper struct for building dynamic types
struct ConvenienceDynamicTypeBuilder {
    builder: DynamicTypeBuilder,
    index: u32,
}

impl ConvenienceDynamicTypeBuilder {
    fn new(name: &str, extensibility: ExtensibilityKind) -> Self {
        Self {
            builder: DynamicTypeBuilderFactory::create_type(TypeDescriptor {
                kind: TypeKind::STRUCTURE,
                name: String::from(name),
                base_type: None,
                discriminator_type: None,
                bound: alloc::vec::Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: extensibility,
                is_nested: false,
            }),
            index: 0,
        }
    }

    fn add_member<T: XTypesBinding>(&mut self, name: &str, id: u32) {
        self.builder
            .add_member(MemberDescriptor {
                name: String::from(name),
                id,
                r#type: T::get_dynamic_type(),
                default_value: None,
                index: self.index,
                try_construct_kind: TryConstructKind::UseDefault,
                label: alloc::vec::Vec::new(),
                is_key: false,
                is_optional: false,
                is_must_understand: true,
                is_shared: false,
                is_default_label: false,
            })
            .unwrap();
        self.index += 1;
    }

    fn build(self) -> DynamicType {
        self.builder.build()
    }
}

impl TypeSupport for TypeLookupRequest {
    fn get_type_name() -> &'static str {
        "TypeLookup_Request"
    }

    fn get_type() -> DynamicType {
        // TypeLookup_Request uses FINAL extensibility per XTypes 1.3
        let mut builder =
            ConvenienceDynamicTypeBuilder::new("TypeLookup_Request", ExtensibilityKind::Final);
        builder.add_member::<RequestHeader>("header", 0);
        // The 'data' field is a union discriminated by hash - we represent it as raw bytes
        // for wire compatibility since union serialization is complex
        builder.add_member::<Vec<u8>>("data", 1);
        builder.build()
    }

    fn create_sample(mut src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        let header =
            RequestHeader::try_from_storage(src.remove_value(0).expect("header must exist"))
                .expect("header must parse");

        // Parse data field (discriminator + payload)
        let data_bytes: Vec<u8> =
            Vec::try_from_storage(src.remove_value(1).expect("data must exist"))
                .expect("data must parse");

        let data = if data_bytes.len() >= 4 {
            let discriminator = u32::from_le_bytes([
                data_bytes[0],
                data_bytes[1],
                data_bytes[2],
                data_bytes[3],
            ]);
            match discriminator {
                TYPELOOKUP_GETTYPES_HASH => {
                    // Parse the getTypes payload:
                    // - 4 bytes: sequence length (number of TypeIdentifiers)
                    // - For each TypeIdentifier: variable length XCDR-encoded
                    let mut type_ids = Vec::new();
                    if data_bytes.len() >= 8 {
                        let seq_len = u32::from_le_bytes([
                            data_bytes[4],
                            data_bytes[5],
                            data_bytes[6],
                            data_bytes[7],
                        ]) as usize;

                        // Parse each TypeIdentifier
                        // TypeIdentifier is a union discriminated by 1-byte kind
                        // For EK_COMPLETE (0xF2) or EK_MINIMAL (0xF1), it's followed by 14-byte hash
                        let mut offset = 8;
                        for _ in 0..seq_len {
                            if offset >= data_bytes.len() {
                                break;
                            }
                            let kind = data_bytes[offset];
                            if kind == 0xF1 || kind == 0xF2 {
                                // Hashed type identifier: 1 byte kind + 14 bytes hash
                                if offset + 15 <= data_bytes.len() {
                                    type_ids.push(data_bytes[offset..offset + 15].to_vec());
                                    offset += 15;
                                } else {
                                    break;
                                }
                            } else {
                                // Other TypeIdentifier kinds - skip for now
                                // In a full implementation, we'd need to parse the full variant
                                break;
                            }
                        }
                    }
                    TypeLookupCall::GetTypes(TypeLookupGetTypesIn { type_ids })
                }
                TYPELOOKUP_GETDEPENDENCIES_HASH => {
                    TypeLookupCall::GetTypeDependencies(TypeLookupGetTypeDependenciesIn::default())
                }
                other => TypeLookupCall::Unknown(other),
            }
        } else {
            TypeLookupCall::default()
        };

        Self { header, data }
    }

    fn create_dynamic_sample(self) -> crate::xtypes::dynamic_type::DynamicData {
        let mut data = DynamicDataFactory::create_data(Self::get_type());
        data.set_value(0, self.header.into_storage());

        // Serialize the call union with discriminator
        let mut call_bytes = Vec::new();
        match &self.data {
            TypeLookupCall::GetTypes(inner) => {
                // Discriminator (4 bytes)
                call_bytes.extend_from_slice(&TYPELOOKUP_GETTYPES_HASH.to_le_bytes());
                // Sequence length (4 bytes)
                call_bytes.extend_from_slice(&(inner.type_ids.len() as u32).to_le_bytes());
                // Each TypeIdentifier as raw bytes
                for type_id in &inner.type_ids {
                    call_bytes.extend_from_slice(type_id);
                }
            }
            TypeLookupCall::GetTypeDependencies(_inner) => {
                call_bytes.extend_from_slice(&TYPELOOKUP_GETDEPENDENCIES_HASH.to_le_bytes());
                // Empty sequence
                call_bytes.extend_from_slice(&0u32.to_le_bytes());
            }
            TypeLookupCall::Unknown(hash) => {
                call_bytes.extend_from_slice(&hash.to_le_bytes());
            }
        }
        data.set_value(1, call_bytes.into_storage());

        data
    }
}

impl TypeSupport for TypeLookupReply {
    fn get_type_name() -> &'static str {
        "TypeLookup_Reply"
    }

    fn get_type() -> DynamicType {
        let mut builder =
            ConvenienceDynamicTypeBuilder::new("TypeLookup_Reply", ExtensibilityKind::Final);
        builder.add_member::<ReplyHeader>("header", 0);
        builder.add_member::<Vec<u8>>("data", 1);
        builder.build()
    }

    fn create_sample(mut src: crate::xtypes::dynamic_type::DynamicData) -> Self {
        let header =
            ReplyHeader::try_from_storage(src.remove_value(0).expect("header must exist"))
                .expect("header must parse");

        let data_bytes: Vec<u8> =
            Vec::try_from_storage(src.remove_value(1).expect("data must exist"))
                .expect("data must parse");

        let data = if data_bytes.len() >= 4 {
            let discriminator = u32::from_le_bytes([
                data_bytes[0],
                data_bytes[1],
                data_bytes[2],
                data_bytes[3],
            ]);
            match discriminator {
                TYPELOOKUP_GETTYPES_HASH => {
                    // Parse TypeLookup_getTypes_Out:
                    // - types: sequence<TypeIdentifierTypeObjectPair>
                    // - complete_to_minimal: sequence<TypeIdentifierPair>
                    let mut cursor = 4usize;

                    // Parse types sequence
                    let mut types = Vec::new();
                    if cursor + 4 <= data_bytes.len() {
                        let types_count = u32::from_le_bytes([
                            data_bytes[cursor],
                            data_bytes[cursor + 1],
                            data_bytes[cursor + 2],
                            data_bytes[cursor + 3],
                        ]) as usize;
                        cursor += 4;

                        for _ in 0..types_count {
                            // Parse type_identifier (sequence<octet>)
                            if cursor + 4 > data_bytes.len() { break; }
                            let ti_len = u32::from_le_bytes([
                                data_bytes[cursor],
                                data_bytes[cursor + 1],
                                data_bytes[cursor + 2],
                                data_bytes[cursor + 3],
                            ]) as usize;
                            cursor += 4;
                            if cursor + ti_len > data_bytes.len() { break; }
                            let type_identifier = data_bytes[cursor..cursor + ti_len].to_vec();
                            cursor += ti_len;

                            // Parse type_object (sequence<octet>)
                            if cursor + 4 > data_bytes.len() { break; }
                            let to_len = u32::from_le_bytes([
                                data_bytes[cursor],
                                data_bytes[cursor + 1],
                                data_bytes[cursor + 2],
                                data_bytes[cursor + 3],
                            ]) as usize;
                            cursor += 4;
                            if cursor + to_len > data_bytes.len() { break; }
                            let type_object = data_bytes[cursor..cursor + to_len].to_vec();
                            cursor += to_len;

                            types.push(TypeIdentifierTypeObjectPairBytes {
                                type_identifier,
                                type_object,
                            });
                        }
                    }

                    // Skip parsing complete_to_minimal for now
                    TypeLookupReturn::GetTypes(TypeLookupGetTypesOut {
                        types,
                        complete_to_minimal: Vec::new(),
                    })
                }
                TYPELOOKUP_GETDEPENDENCIES_HASH => {
                    TypeLookupReturn::GetTypeDependencies(TypeLookupGetTypeDependenciesOut::default())
                }
                other => TypeLookupReturn::Unknown(other, data_bytes[4..].to_vec()),
            }
        } else {
            TypeLookupReturn::default()
        };

        Self { header, data }
    }

    fn create_dynamic_sample(self) -> crate::xtypes::dynamic_type::DynamicData {
        let mut data = DynamicDataFactory::create_data(Self::get_type());
        data.set_value(0, self.header.into_storage());

        let mut return_bytes = Vec::new();
        match &self.data {
            TypeLookupReturn::GetTypes(inner) => {
                // Format: discriminator (4 bytes) + TypeLookup_getTypes_Out
                return_bytes.extend_from_slice(&TYPELOOKUP_GETTYPES_HASH.to_le_bytes());

                // Serialize types sequence: length (4 bytes) + items
                return_bytes.extend_from_slice(&(inner.types.len() as u32).to_le_bytes());
                for pair in &inner.types {
                    // Each TypeIdentifierTypeObjectPair:
                    // - type_identifier: sequence<octet> (4-byte length + bytes)
                    // - type_object: sequence<octet> (4-byte length + bytes)
                    return_bytes
                        .extend_from_slice(&(pair.type_identifier.len() as u32).to_le_bytes());
                    return_bytes.extend_from_slice(&pair.type_identifier);
                    return_bytes.extend_from_slice(&(pair.type_object.len() as u32).to_le_bytes());
                    return_bytes.extend_from_slice(&pair.type_object);
                }

                // Serialize complete_to_minimal sequence (empty for now)
                return_bytes
                    .extend_from_slice(&(inner.complete_to_minimal.len() as u32).to_le_bytes());
                for pair in &inner.complete_to_minimal {
                    return_bytes
                        .extend_from_slice(&(pair.type_identifier1.len() as u32).to_le_bytes());
                    return_bytes.extend_from_slice(&pair.type_identifier1);
                    return_bytes
                        .extend_from_slice(&(pair.type_identifier2.len() as u32).to_le_bytes());
                    return_bytes.extend_from_slice(&pair.type_identifier2);
                }
            }
            TypeLookupReturn::GetTypeDependencies(inner) => {
                return_bytes.extend_from_slice(&TYPELOOKUP_GETDEPENDENCIES_HASH.to_le_bytes());

                // Serialize dependent_typeids sequence
                return_bytes
                    .extend_from_slice(&(inner.dependent_typeids.len() as u32).to_le_bytes());
                for type_id_with_size in &inner.dependent_typeids {
                    return_bytes
                        .extend_from_slice(&(type_id_with_size.type_id.len() as u32).to_le_bytes());
                    return_bytes.extend_from_slice(&type_id_with_size.type_id);
                    return_bytes.extend_from_slice(
                        &type_id_with_size.typeobject_serialized_size.to_le_bytes(),
                    );
                }

                // Serialize continuation_point sequence
                return_bytes
                    .extend_from_slice(&(inner.continuation_point.len() as u32).to_le_bytes());
                return_bytes.extend_from_slice(&inner.continuation_point);
            }
            TypeLookupReturn::Unknown(hash, bytes) => {
                return_bytes.extend_from_slice(&hash.to_le_bytes());
                return_bytes.extend_from_slice(bytes);
            }
        }
        data.set_value(1, return_bytes.into_storage());

        data
    }
}

// ============================================================================
// Builtin Endpoint Bits for SPDP (from Fast-DDS BuiltinEndpoints.hpp)
// ============================================================================

/// Bit for TypeLookup service request writer
pub const BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REQUEST_DATA_WRITER: u32 = 1 << 12; // 0x00001000

/// Bit for TypeLookup service request reader
pub const BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REQUEST_DATA_READER: u32 = 1 << 13; // 0x00002000

/// Bit for TypeLookup service reply writer
pub const BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REPLY_DATA_WRITER: u32 = 1 << 14; // 0x00004000

/// Bit for TypeLookup service reply reader
pub const BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REPLY_DATA_READER: u32 = 1 << 15; // 0x00008000

/// Combined mask for all TypeLookup endpoints
pub const BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_ALL: u32 =
    BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REQUEST_DATA_WRITER
        | BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REQUEST_DATA_READER
        | BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REPLY_DATA_WRITER
        | BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REPLY_DATA_READER;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_identity_roundtrip() {
        let identity = SampleIdentity {
            writer_guid: Guid::new(
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
                crate::transport::types::EntityId::new([0, 0, 1], 0xc2),
            ),
            sequence_number: 42,
        };
        let storage = identity.clone().into_storage();
        let recovered = SampleIdentity::try_from_storage(storage).unwrap();
        assert_eq!(identity, recovered);
    }

    #[test]
    fn test_request_header_roundtrip() {
        let header = RequestHeader {
            request_id: SampleIdentity::default(),
            instance_name: String::from("test_service"),
        };
        let storage = header.clone().into_storage();
        let recovered = RequestHeader::try_from_storage(storage).unwrap();
        assert_eq!(header, recovered);
    }

    #[test]
    fn test_reply_header_roundtrip() {
        let header = ReplyHeader {
            related_request_id: SampleIdentity::default(),
            remote_ex: REMOTE_EX_OK,
        };
        let storage = header.clone().into_storage();
        let recovered = ReplyHeader::try_from_storage(storage).unwrap();
        assert_eq!(header, recovered);
    }

    #[test]
    fn test_operation_hash_values() {
        // Verify hash values match XTypes 1.3 spec
        assert_eq!(TYPELOOKUP_GETTYPES_HASH, 0x018252d3);
        assert_eq!(TYPELOOKUP_GETDEPENDENCIES_HASH, 0x05aafb31);
    }

    #[test]
    fn test_builtin_endpoint_bits() {
        // Verify bit positions match Fast-DDS
        assert_eq!(
            BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REQUEST_DATA_WRITER,
            0x00001000
        );
        assert_eq!(
            BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REQUEST_DATA_READER,
            0x00002000
        );
        assert_eq!(
            BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REPLY_DATA_WRITER,
            0x00004000
        );
        assert_eq!(
            BUILTIN_ENDPOINT_TYPELOOKUP_SERVICE_REPLY_DATA_READER,
            0x00008000
        );
    }
}
