use alloc::{boxed::Box, string::String, vec::Vec};
use dust_dds_derive::DdsType;

use crate::xtypes::{
    dynamic_type::{DynamicDataFactory, DynamicType, DynamicTypeMember},
    error::XTypesError,
    serializer::serialize_cdr2_le,
    type_support::TypeSupport,
};

use super::dynamic_type::{ExtensibilityKind, TryConstructKind, TypeKind};

/* Manually created from dds-xtypes_typeobject.idl */

// ---------- Equivalence Kinds -------------------
pub type EquivalenceKind = u8;
pub const EK_MINIMAL: EquivalenceKind = 0xF1; // 0x1111 0001
pub const EK_COMPLETE: EquivalenceKind = 0xF2; // 0x1111 0010
pub const EK_BOTH: EquivalenceKind = 0xF3; // 0x1111 0011

// ---------- TypeKinds (begin) -------------------
// Primitive TKs
pub const TK_NONE: u8 = 0x00;
pub const TK_BOOLEAN: u8 = 0x01;
pub const TK_BYTE: u8 = 0x02;
pub const TK_INT16: u8 = 0x03;
pub const TK_INT32: u8 = 0x04;
pub const TK_INT64: u8 = 0x05;
pub const TK_UINT16: u8 = 0x06;
pub const TK_UINT32: u8 = 0x07;
pub const TK_UINT64: u8 = 0x08;
pub const TK_FLOAT32: u8 = 0x09;
pub const TK_FLOAT64: u8 = 0x0A;
pub const TK_FLOAT128: u8 = 0x0B;
pub const TK_INT8: u8 = 0x0C;
pub const TK_UINT8: u8 = 0x0D;
pub const TK_CHAR8: u8 = 0x10;
pub const TK_CHAR16: u8 = 0x11;

// String TKs
pub const TK_STRING8: u8 = 0x20;
pub const TK_STRING16: u8 = 0x21;

// Constructed/Named types
pub const TK_ALIAS: u8 = 0x30;

// Enumerated TKs
pub const TK_ENUM: u8 = 0x40;
pub const TK_BITMASK: u8 = 0x41;

// Structured TKs
pub const TK_ANNOTATION: u8 = 0x50;
pub const TK_STRUCTURE: u8 = 0x51;
pub const TK_UNION: u8 = 0x52;
pub const TK_BITSET: u8 = 0x53;

// Collection TKs
pub const TK_SEQUENCE: u8 = 0x60;
pub const TK_ARRAY: u8 = 0x61;
pub const TK_MAP: u8 = 0x62;
// ---------- TypeKinds (end) -------------------

// ---------- Extra TypeIdentifiers (begin) ------------
pub type TypeIdentiferKind = u8;
pub const TI_STRING8_SMALL: u8 = 0x70;
pub const TI_STRING8_LARGE: u8 = 0x71;
pub const TI_STRING16_SMALL: u8 = 0x72;
pub const TI_STRING16_LARGE: u8 = 0x73;
pub const TI_PLAIN_SEQUENCE_SMALL: u8 = 0x80;
pub const TI_PLAIN_SEQUENCE_LARGE: u8 = 0x81;
pub const TI_PLAIN_ARRAY_SMALL: u8 = 0x90;
pub const TI_PLAIN_ARRAY_LARGE: u8 = 0x91;
pub const TI_PLAIN_MAP_SMALL: u8 = 0xA0;
pub const TI_PLAIN_MAP_LARGE: u8 = 0xA1;
pub const TI_STRONGLY_CONNECTED_COMPONENT: u8 = 0xB0;
// ---------- Extra TypeIdentifiers (end) --------------

// The name of some element (e.g. type, type member, module)
// Valid characters are alphanumeric plus the "_" cannot start with digit
pub const MEMBER_NAME_MAX_LENGTH: i32 = 256;
pub type MemberName = String; //string<MEMBER_NAME_MAX_LENGTH>

// Qualified type name includes the name of containing modules
// using "::" as separator. No leading "::". E.g. "MyModule::MyType"
pub const TYPE_NAME_MAX_LENGTH: i32 = 256;
pub type QualifiedTypeName = String; //string<TYPE_NAME_MAX_LENGTH>

// Every type has an ID. Those of the primitive types are pre-defined.
pub type PrimitiveTypeId = u8;

// First 14 bytes of MD5 of the serialized TypeObject using XCDR
// version 2 with Little Endian encoding
pub type EquivalenceHash = [u8; 14];

// First 4 bytes of MD5 of of a member name converted to bytes
// using UTF-8 encoding and without a 'nul' terminator.
// Example: the member name "color" has NameHash {0x70, 0xDD, 0xA5, 0xDF}
pub type NameHash = [u8; 4];

// Long Bound of a collection type
pub type LBound = u32;
pub type LBoundSeq = Vec<LBound>;
pub const INVALID_LBOUND: LBound = 0;

// Short Bound of a collection type
pub type SBound = u8;
pub type SBoundSeq = Vec<SBound>;
pub const INVALID_SBOUND: SBound = 0;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested, switch(u8))]
pub enum TypeObjectHashId {
    #[dust_dds(case=EK_COMPLETE)]
    EkComplete { hash: EquivalenceHash },
    #[dust_dds(case=EK_MINIMAL)]
    EkMinimal { hash: EquivalenceHash },
}

// Flags that apply to struct/union/collection/enum/bitmask/bitset
// members/elements and DO affect type assignability
// Depending on the flag it may not apply to members of all types
// When not all, the applicable member types are listed

#[derive(Debug, Clone, Copy, DdsType)]
pub struct MemberFlag(u16);

impl PartialEq for MemberFlag {
    fn eq(&self, other: &Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for MemberFlag {
    type Output = MemberFlag;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for MemberFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub const MEMBER_FLAG_TRY_CONSTRUCT1: MemberFlag = MemberFlag(1 << 0); // T1 | 00 = INVALID, 01 = DISCARD
pub const MEMBER_FLAG_TRY_CONSTRUCT2: MemberFlag = MemberFlag(1 << 1); // T2 | 10 = USE_DEFAULT, 11 = TRIM
pub const MEMBER_FLAG_IS_EXTERNAL: MemberFlag = MemberFlag(1 << 2); // X StructMember, UnionMember,
// CollectionElement
pub const MEMBER_FLAG_IS_OPTIONAL: MemberFlag = MemberFlag(1 << 3); // O StructMember
pub const MEMBER_FLAG_IS_MUST_UNDERSTAND: MemberFlag = MemberFlag(1 << 4); // M StructMember
pub const MEMBER_FLAG_IS_KEY: MemberFlag = MemberFlag(1 << 5); // K StructMember, UnionDiscriminator
pub const MEMBER_FLAG_IS_DEFAULT: MemberFlag = MemberFlag(1 << 6); // D UnionMember, EnumerationLiteral

pub type CollectionElementFlag = MemberFlag; // T1, T2, X

pub type StructMemberFlag = MemberFlag; // T1, T2, O, M, K, X
pub type UnionMemberFlag = MemberFlag; // T1, T2, D, X
pub type UnionDiscriminatorFlag = MemberFlag; // T1, T2, K
pub type EnumeratedLiteralFlag = MemberFlag; // D
pub type AnnotationParameterFlag = MemberFlag; // Unused. No flags apply
pub type AliasMemberFlag = MemberFlag; // Unused. No flags apply
pub type BitflagFlag = MemberFlag; // Unused. No flags apply
pub type BitsetMemberFlag = MemberFlag; // Unused. No flags apply

// Mask used to remove the flags that do no affect assignability
// Selects T1, T2, O, M, K, D
pub const MEMBER_FLAG_MINIMAL_MASK: MemberFlag = MemberFlag(0x003f);
// Flags that apply to type declarationa and DO affect assignability
// Depending on the flag it may not apply to all types
// When not all, the applicable types are listed

#[derive(DdsType, Debug, Clone)]
#[dust_dds(extensibility = "final", nested)]
pub struct TypeFlag(u16);

impl PartialEq for TypeFlag {
    fn eq(&self, other: &Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for TypeFlag {
    type Output = TypeFlag;
    fn bitor(self, rhs: TypeFlag) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for TypeFlag {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub const TYPE_FLAG_IS_FINAL: TypeFlag = TypeFlag(1 << 0); // F
pub const TYPE_FLAG_IS_APPENDABLE: TypeFlag = TypeFlag(1 << 1); // A |- Struct, Union
pub const TYPE_FLAG_IS_MUTABLE: TypeFlag = TypeFlag(1 << 2); // M | (exactly one flag)
pub const TYPE_FLAG_IS_NESTED: TypeFlag = TypeFlag(1 << 3); // N Struct, Union
pub const TYPE_FLAG_IS_AUTOID_HASH: TypeFlag = TypeFlag(1 << 4); // H Struct

pub type StructTypeFlag = TypeFlag; // All flags apply
pub type UnionTypeFlag = TypeFlag; // All flags apply
pub type CollectionTypeFlag = TypeFlag; // Unused. No flags apply
pub type AnnotationTypeFlag = TypeFlag; // Unused. No flags apply
pub type AliasTypeFlag = TypeFlag; // Unused. No flags apply
pub type EnumTypeFlag = TypeFlag; // Unused. No flags apply
pub type BitmaskTypeFlag = TypeFlag; // Unused. No flags apply
pub type BitsetTypeFlag = TypeFlag; // Unused. No flags apply

// Mask used to remove the flags that do no affect assignability
pub const TYPE_FLAG_MINIMAL_MASK: TypeFlag = TypeFlag(0x0007); // Selects M, A, F

// 1 Byte
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct StringSTypeDefn {
    pub bound: SBound,
}
// 4 Bytes
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct StringLTypeDefn {
    pub bound: LBound,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainCollectionHeader {
    pub equiv_kind: EquivalenceKind,
    pub element_flags: CollectionElementFlag,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainSequenceSElemDefn {
    pub header: PlainCollectionHeader,
    pub bound: SBound,
    #[dust_dds(external)]
    pub element_identifier: Box<TypeIdentifier>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainSequenceLElemDefn {
    pub header: PlainCollectionHeader,
    pub bound: LBound,
    #[dust_dds(external)]
    pub element_identifier: Box<TypeIdentifier>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainArraySElemDefn {
    pub header: PlainCollectionHeader,
    pub array_bound_seq: SBoundSeq,
    #[dust_dds(external)]
    pub element_identifier: Box<TypeIdentifier>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainArrayLElemDefn {
    pub header: PlainCollectionHeader,
    pub array_bound_seq: LBoundSeq,
    #[dust_dds(external)]
    pub element_identifier: Box<TypeIdentifier>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainMapSTypeDefn {
    pub header: PlainCollectionHeader,
    pub bound: SBound,
    #[dust_dds(external)]
    pub element_identifier: Box<TypeIdentifier>,
    pub key_flags: CollectionElementFlag,
    #[dust_dds(external)]
    pub key_identifier: Box<TypeIdentifier>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct PlainMapLTypeDefn {
    pub header: PlainCollectionHeader,
    pub bound: LBound,
    #[dust_dds(external)]
    pub element_identifier: Box<TypeIdentifier>,
    pub key_flags: CollectionElementFlag,
    #[dust_dds(external)]
    pub key_identifier: Box<TypeIdentifier>,
}

// Used for Types that have cyclic depencencies with other types
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct StronglyConnectedComponentId {
    pub sc_component_id: TypeObjectHashId, // Hash StronglyConnectedComponent
    pub scc_length: i32,                   // StronglyConnectedComponent.length
    pub scc_index: i32,                    // identify type in Strongly Connected Comp.
}
// Future extensibility
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "mutable", nested)]
pub struct ExtendedTypeDefn {
    // Empty. Available for future extension
}

// The TypeIdentifier uniquely identifies a type (a set of equivalent
// types according to an equivalence relationship: COMPLETE, MNIMAL).
//
// In some cases (primitive types, strings, plain types) the identifier
// is a explicit description of the type.
// In other cases the Identifier is a Hash of the type description
//
// In the of primitive types and strings the implied equivalence
// relation is the identity.
//
// For Plain Types and Hash-defined TypeIdentifiers there are three
// possibilities: MINIMAL, COMPLETE, and COMMON:
// - MINIMAL indicates the TypeIdentifier identifies equivalent types
// according to the MINIMAL equivalence relation
// - COMPLETE indicates the TypeIdentifier identifies equivalent types
// according to the COMPLETE equivalence relation
// - COMMON indicates the TypeIdentifier identifies equivalent types
// according to both the MINIMAL and the COMMON equivalence relation.
// This means the TypeIdentifier is the same for both relationships
//
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested, switch(u8))]
pub enum TypeIdentifier {
    #[dust_dds(case=TK_NONE)]
    TkNone,
    #[dust_dds(case=TK_BOOLEAN)]
    TkBoolean,
    #[dust_dds(case=TK_BYTE)]
    TkByteType,
    #[dust_dds(case=TK_INT8)]
    TkInt8Type,
    #[dust_dds(case=TK_INT16)]
    TkInt16Type,
    #[dust_dds(case=TK_INT32)]
    TkInt32Type,
    #[dust_dds(case=TK_INT64)]
    TkInt64Type,
    #[dust_dds(case=TK_UINT8)]
    TkUint8Type,
    #[dust_dds(case=TK_UINT16)]
    TkUint16Type,
    #[dust_dds(case=TK_UINT32)]
    TkUint32Type,
    #[dust_dds(case=TK_UINT64)]
    TkUint64Type,
    #[dust_dds(case=TK_FLOAT32)]
    TkFloat32Type,
    #[dust_dds(case=TK_FLOAT64)]
    TkFloat64Type,
    #[dust_dds(case=TK_FLOAT128)]
    TkFloat128Type,
    #[dust_dds(case=TK_CHAR8)]
    TkChar8Type,
    #[dust_dds(case=TK_CHAR16)]
    TkChar16Type,
    // ============ Strings - use TypeIdentifierKind ===================
    #[dust_dds(case=TI_STRING8_SMALL)]
    TiString8Small { string_sdefn: StringSTypeDefn },
    #[dust_dds(case=TI_STRING16_SMALL)]
    TiString16Small { string_sdefn: StringSTypeDefn },
    #[dust_dds(case=TI_STRING8_LARGE)]
    TiString8Large { string_ldefn: StringLTypeDefn },
    #[dust_dds(case=TI_STRING16_LARGE)]
    TiString16Large { string_ldefn: StringLTypeDefn },
    // ============ Plain collectios - use TypeIdentifierKind =========
    #[dust_dds(case=TI_PLAIN_SEQUENCE_SMALL)]
    TiPlainSequenceSmall { seq_sdefn: PlainSequenceSElemDefn },
    #[dust_dds(case=TI_PLAIN_SEQUENCE_LARGE)]
    TiPlainSequenceLarge { seq_ldefn: PlainSequenceLElemDefn },
    #[dust_dds(case=TI_PLAIN_ARRAY_SMALL)]
    TiPlainArraySmall { array_sdefn: PlainArraySElemDefn },
    #[dust_dds(case=TI_PLAIN_ARRAY_LARGE)]
    TiPlainArrayLarge { array_ldefn: PlainArrayLElemDefn },
    #[dust_dds(case=TI_PLAIN_MAP_SMALL)]
    TiPlainMapSmall { map_sdefn: PlainMapSTypeDefn },
    #[dust_dds(case=TI_PLAIN_MAP_LARGE)]
    TiPlainMapLarge { map_ldefn: PlainMapLTypeDefn },
    // ============ Types that are mutually dependent on each other ===
    #[dust_dds(case=TI_STRONGLY_CONNECTED_COMPONENT)]
    TiStronglyConnectedComponent {
        sc_component_id: StronglyConnectedComponentId,
    },
    // ============ The remaining cases - use EquivalenceKind =========
    #[dust_dds(case=EK_COMPLETE)]
    EkComplete { equivalence_hash: EquivalenceHash },
    #[dust_dds(case=EK_MINIMAL)]
    EkMinimal { equivalence_hash: EquivalenceHash },
    // =================== Future extensibility ============
    #[dust_dds(default)]
    Default { extended_type: MinimalExtendedType },
}

impl Default for TypeIdentifier {
    fn default() -> Self {
        TypeIdentifier::Default {
            extended_type: Default::default(),
        }
    }
}

pub type TypeIdentifierSeq = Vec<TypeIdentifier>;

// --- Annotation usage: -----------------------------------------------
// ID of a type member
pub type MemberId = u32;
pub const ANNOTATION_STR_VALUE_MAX_LEN: u32 = 128;
pub const ANNOTATION_OCTETSEC_VALUE_MAX_LEN: u32 = 128;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "mutable", nested)]
pub struct ExtendedAnnotationParameterValue {
    // Empty. Available for future extension
}

/* Literal value of an annotation member: either the default value in its
* definition or the value applied in its usage.
*/
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested, switch(u8))]
pub enum AnnotationParameterValue {
    #[dust_dds(case=TK_BOOLEAN)]
    TkBoolean { boolean_value: bool },
    #[dust_dds(case=TK_BYTE)]
    TkByte { byte_value: u8 },
    #[dust_dds(case=TK_INT8)]
    TkInt8 { int8_value: i8 },
    #[dust_dds(case=TK_UINT8)]
    TkUint8 { uint8_value: u8 },
    #[dust_dds(case=TK_INT16)]
    TkInt16 { int16_value: i16 },
    #[dust_dds(case=TK_UINT16)]
    TkUint16 { uint_16_value: u16 },
    #[dust_dds(case=TK_INT32)]
    TkInt32 { int32_value: i32 },
    #[dust_dds(case=TK_UINT32)]
    TkUint32 { uint32_value: u32 },
    #[dust_dds(case=TK_INT64)]
    TkInt64 { int64_value: i64 },
    #[dust_dds(case=TK_UINT64)]
    TkUint64 { uint64_value: u64 },
    #[dust_dds(case=TK_FLOAT32)]
    TkFloat32 { float32_value: f32 },
    #[dust_dds(case=TK_FLOAT64)]
    TkFloat64 { float64_value: f64 },
    // TkFloat128 {
    //     float128_value: i128,
    // },
    #[dust_dds(case=TK_CHAR8)]
    TkChar8 { char_value: char },
    // TypeKind::CHAR16 {
    // wchar_value: char16},
    #[dust_dds(case=TK_ENUM)]
    TkEnum { enumerated_value: i32 },
    #[dust_dds(case=TK_STRING8)]
    TkString8 {
        string8_value: String, /*string<ANNOTATION_STR_VALUE_MAX_LEN>  */
    },
    // TypeKind::STRING16:
    // wstring<ANNOTATION_STR_VALUE_MAX_LEN> string16_value;
    #[dust_dds(default)]
    Default {
        extended_value: ExtendedAnnotationParameterValue,
    },
}

// The application of an annotation to some type or type member
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct AppliedAnnotationParameter {
    pub paramname_hash: NameHash,
    pub value: AnnotationParameterValue,
}

// Sorted by AppliedAnnotationParameter.paramname_hash
pub type AppliedAnnotationParameterSeq = Vec<AppliedAnnotationParameter>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct AppliedAnnotation {
    pub annotation_typeid: TypeIdentifier,
    #[dust_dds(optional)]
    pub param_seq: Option<AppliedAnnotationParameterSeq>,
}
// Sorted by AppliedAnnotation.annotation_typeid
pub type AppliedAnnotationSeq = Vec<AppliedAnnotation>;
// @verbatim(placement="<placement>", language="<lang>", text="<text>")
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct AppliedVerbatimAnnotation {
    pub placement: String, //string<32>
    pub language: String,  //string<32>
    pub text: String,
}

// --- Aggregate types: ------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct AppliedBuiltinMemberAnnotations {
    #[dust_dds(optional)]
    pub unit: Option<String>, // @unit("<unit>")
    #[dust_dds(optional)]
    pub min: Option<AnnotationParameterValue>, // @min , @range
    #[dust_dds(optional)]
    pub max: Option<AnnotationParameterValue>, // @max , @range
    #[dust_dds(optional)]
    pub hash_id: Option<String>, // @hashid("<membername>")
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonStructMember {
    pub member_id: MemberId,
    pub member_flags: StructMemberFlag,
    pub member_type_id: TypeIdentifier,
}

// COMPLETE Details for a member of an aggregate type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteMemberDetail {
    pub name: MemberName,
    #[dust_dds(optional)]
    pub ann_builtin: Option<AppliedBuiltinMemberAnnotations>,
    #[dust_dds(optional)]
    pub ann_custom: Option<AppliedAnnotationSeq>,
}
// MINIMAL Details for a member of an aggregate type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalMemberDetail {
    pub name_hash: NameHash,
}

// Member of an aggregate type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteStructMember {
    pub common: CommonStructMember,
    pub detail: CompleteMemberDetail,
}
// Ordered by the member_index
pub type CompleteStructMemberSeq = Vec<CompleteStructMember>;
// Member of an aggregate type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalStructMember {
    pub common: CommonStructMember,
    pub detail: MinimalMemberDetail,
}
// Ordered by common.member_id
pub type MinimalStructMemberSeq = Vec<MinimalStructMember>;
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct AppliedBuiltinTypeAnnotations {
    #[dust_dds(optional)]
    pub verbatim: Option<AppliedVerbatimAnnotation>, // @verbatim(...)
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalTypeDetail {
    // Empty. Available for future extension
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteTypeDetail {
    #[dust_dds(optional)]
    pub ann_builtin: Option<AppliedBuiltinTypeAnnotations>,
    #[dust_dds(optional)]
    pub ann_custom: Option<AppliedAnnotationSeq>,
    pub type_name: QualifiedTypeName,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteStructHeader {
    pub base_type: TypeIdentifier,
    pub detail: CompleteTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalStructHeader {
    pub base_type: TypeIdentifier,
    pub detail: MinimalTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteStructType {
    pub struct_flags: StructTypeFlag,
    pub header: CompleteStructHeader,
    pub member_seq: CompleteStructMemberSeq,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalStructType {
    pub struct_flags: StructTypeFlag,
    pub header: MinimalStructHeader,
    pub member_seq: MinimalStructMemberSeq,
}

// --- Union: ----------------------------------------------------------
// labels that apply to a member of a union type
// Ordered by their values
pub type UnionCaseLabelSeq = Vec<i32>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonUnionMember {
    pub member_id: MemberId,
    pub member_flags: UnionMemberFlag,
    pub type_id: TypeIdentifier,
    pub label_seq: UnionCaseLabelSeq,
}

// Member of a union type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteUnionMember {
    pub common: CommonUnionMember,
    pub detail: CompleteMemberDetail,
}
// Ordered by member_index
pub type CompleteUnionMemberSeq = Vec<CompleteUnionMember>;

// Member of a union type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalUnionMember {
    pub common: CommonUnionMember,
    pub detail: MinimalMemberDetail,
}
// Ordered by MinimalUnionMember.common.member_id
pub type MinimalUnionMemberSeq = Vec<MinimalUnionMember>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonDiscriminatorMember {
    pub member_flags: UnionDiscriminatorFlag,
    pub type_id: TypeIdentifier,
}
// Member of a union type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteDiscriminatorMember {
    pub common: CommonDiscriminatorMember,
    #[dust_dds(optional)]
    pub ann_builtin: Option<AppliedBuiltinTypeAnnotations>,
    #[dust_dds(optional)]
    pub ann_custom: Option<AppliedAnnotationSeq>,
}
// Member of a union type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalDiscriminatorMember {
    pub common: CommonDiscriminatorMember,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteUnionHeader {
    pub detail: CompleteTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalUnionHeader {
    pub detail: MinimalTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteUnionType {
    pub union_flags: UnionTypeFlag,
    pub header: CompleteUnionHeader,
    pub discriminator: CompleteDiscriminatorMember,
    pub member_seq: CompleteUnionMemberSeq,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalUnionType {
    pub union_flags: UnionTypeFlag,
    pub header: MinimalUnionHeader,
    pub discriminator: MinimalDiscriminatorMember,
    pub member_seq: MinimalUnionMemberSeq,
}

// --- Annotation: ----------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonAnnotationParameter {
    pub member_flags: AnnotationParameterFlag,
    pub member_type_id: TypeIdentifier,
}

// Member of an annotation type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteAnnotationParameter {
    pub common: CommonAnnotationParameter,
    pub name: MemberName,
    pub default_value: AnnotationParameterValue,
}
// Ordered by CompleteAnnotationParameter.name
pub type CompleteAnnotationParameterSeq = Vec<CompleteAnnotationParameter>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalAnnotationParameter {
    pub common: CommonAnnotationParameter,
    pub name_hash: NameHash,
    pub default_value: AnnotationParameterValue,
}
// Ordered by MinimalAnnotationParameter.name_hash
pub type MinimalAnnotationParameterSeq = Vec<MinimalAnnotationParameter>;
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteAnnotationHeader {
    pub annotation_name: QualifiedTypeName,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalAnnotationHeader {
    // Empty. Available for future extension
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteAnnotationType {
    pub annotation_flag: AnnotationTypeFlag,
    pub header: CompleteAnnotationHeader,
    pub member_seq: CompleteAnnotationParameterSeq,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalAnnotationType {
    pub annotation_flag: AnnotationTypeFlag,
    pub header: MinimalAnnotationHeader,
    pub member_seq: MinimalAnnotationParameterSeq,
}

// --- Alias: ----------------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonAliasBody {
    pub related_flags: AliasMemberFlag,
    pub related_type: TypeIdentifier,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteAliasBody {
    pub common: CommonAliasBody,
    #[dust_dds(optional)]
    pub ann_builtin: Option<AppliedBuiltinMemberAnnotations>,
    #[dust_dds(optional)]
    pub ann_custom: Option<AppliedAnnotationSeq>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalAliasBody {
    pub common: CommonAliasBody,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteAliasHeader {
    pub detail: CompleteTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalAliasHeader {
    // Empty. Available for future extension
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteAliasType {
    pub alias_flags: AliasTypeFlag,
    pub header: CompleteAliasHeader,
    pub body: CompleteAliasBody,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalAliasType {
    pub alias_flags: AliasTypeFlag,
    pub header: MinimalAliasHeader,
    pub body: MinimalAliasBody,
}

// --- Collections: ----------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteElementDetail {
    #[dust_dds(optional)]
    pub ann_builtin: Option<AppliedBuiltinMemberAnnotations>,
    #[dust_dds(optional)]
    pub ann_custom: Option<AppliedAnnotationSeq>,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonCollectionElement {
    pub element_flags: CollectionElementFlag,
    pub _type: TypeIdentifier,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteCollectionElement {
    pub common: CommonCollectionElement,
    pub detail: CompleteElementDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalCollectionElement {
    pub common: CommonCollectionElement,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonCollectionHeader {
    pub bound: LBound,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteCollectionHeader {
    pub common: CommonCollectionHeader,
    #[dust_dds(optional)]
    pub detail: Option<CompleteTypeDetail>, // not present for anonymous
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalCollectionHeader {
    pub common: CommonCollectionHeader,
}

// --- Sequence: ------------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteSequenceType {
    pub collection_flag: CollectionTypeFlag,
    pub header: CompleteCollectionHeader,
    pub element: CompleteCollectionElement,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalSequenceType {
    pub collection_flag: CollectionTypeFlag,
    pub header: MinimalCollectionHeader,
    pub element: MinimalCollectionElement,
}

// --- Array: ------------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonArrayHeader {
    pub bound_seq: LBoundSeq,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteArrayHeader {
    pub common: CommonArrayHeader,
    pub detail: CompleteTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalArrayHeader {
    pub common: CommonArrayHeader,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteArrayType {
    pub collection_flag: CollectionTypeFlag,
    pub header: CompleteArrayHeader,
    pub element: CompleteCollectionElement,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalArrayType {
    pub collection_flag: CollectionTypeFlag,
    pub header: MinimalArrayHeader,
    pub element: MinimalCollectionElement,
}

// --- Map: ------------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteMapType {
    pub collection_flag: CollectionTypeFlag,
    pub header: CompleteCollectionHeader,
    pub key: CompleteCollectionElement,
    pub element: CompleteCollectionElement,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalMapType {
    pub collection_flag: CollectionTypeFlag,
    pub header: MinimalCollectionHeader,
    pub key: MinimalCollectionElement,
    pub element: MinimalCollectionElement,
}

// --- Enumeration: ----------------------------------------------------
pub type BitBound = u16;
// Constant in an enumerated type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CommonEnumeratedLiteral {
    pub value: i32,
    pub flags: EnumeratedLiteralFlag,
}

// Constant in an enumerated type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteEnumeratedLiteral {
    pub common: CommonEnumeratedLiteral,
    pub detail: CompleteMemberDetail,
}

// Ordered by EnumeratedLiteral.common.value
pub type CompleteEnumeratedLiteralSeq = Vec<CompleteEnumeratedLiteral>;

// Constant in an enumerated type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalEnumeratedLiteral {
    pub common: CommonEnumeratedLiteral,
    pub detail: MinimalMemberDetail,
}

// Ordered by EnumeratedLiteral.common.value
pub type MinimalEnumeratedLiteralSeq = Vec<MinimalEnumeratedLiteral>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonEnumeratedHeader {
    pub bit_bound: BitBound,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteEnumeratedHeader {
    pub common: CommonEnumeratedHeader,
    pub detail: CompleteTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalEnumeratedHeader {
    pub common: CommonEnumeratedHeader,
}

// Enumerated type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CompleteEnumeratedType {
    pub enum_flags: EnumTypeFlag, // unused
    pub header: CompleteEnumeratedHeader,
    pub literal_seq: CompleteEnumeratedLiteralSeq,
}

// Enumerated type
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct MinimalEnumeratedType {
    pub enum_flags: EnumTypeFlag, // unused
    pub header: MinimalEnumeratedHeader,
    pub literal_seq: MinimalEnumeratedLiteralSeq,
}

// --- Bitmask: --------------------------------------------------------
// Bit in a bit mask
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonBitflag {
    pub position: u16,
    pub flags: BitflagFlag,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteBitflag {
    pub common: CommonBitflag,
    pub detail: CompleteMemberDetail,
}
// Ordered by Bitflag.position
pub type CompleteBitflagSeq = Vec<CompleteBitflag>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalBitflag {
    pub common: CommonBitflag,
    pub detail: MinimalMemberDetail,
}
// Ordered by Bitflag.position
pub type MinimalBitflagSeq = Vec<MinimalBitflag>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonBitmaskHeader {
    pub bit_bound: BitBound,
}
pub type CompleteBitmaskHeader = CompleteEnumeratedHeader;
pub type MinimalBitmaskHeader = MinimalEnumeratedHeader;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteBitmaskType {
    pub bitmask_flags: BitmaskTypeFlag, // unused
    pub header: CompleteBitmaskHeader,
    pub flag_seq: CompleteBitflagSeq,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalBitmaskType {
    pub bitmask_flags: BitmaskTypeFlag, // unused
    pub header: MinimalBitmaskHeader,
    pub flag_seq: MinimalBitflagSeq,
}

// --- Bitset: ----------------------------------------------------------
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct CommonBitfield {
    pub position: u16,
    pub flags: BitsetMemberFlag,
    pub bitcount: u8,
    pub holder_type: u8, // Original in IDL: TypeKind which for us is a Rust enum but in IDL is an octet // Must be primitive integer type
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteBitfield {
    pub common: CommonBitfield,
    pub detail: CompleteMemberDetail,
}

// Ordered by Bitfield.position
pub type CompleteBitfieldSeq = Vec<CompleteBitfield>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalBitfield {
    pub common: CommonBitfield,
    pub name_hash: NameHash,
}

// Ordered by Bitfield.position
pub type MinimalBitfieldSeq = Vec<MinimalBitfield>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteBitsetHeader {
    pub detail: CompleteTypeDetail,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalBitsetHeader {
    // Empty. Available for future extension
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct CompleteBitsetType {
    pub bitset_flags: BitsetTypeFlag, // unused
    pub header: CompleteBitsetHeader,
    pub field_seq: CompleteBitfieldSeq,
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct MinimalBitsetType {
    pub bitset_flags: BitsetTypeFlag, // unused
    pub header: MinimalBitsetHeader,
    pub field_seq: MinimalBitfieldSeq,
}

// --- Type Object: ---------------------------------------------------
// The types associated with each selection must have extensibility
// kind APPENDABLE or MUTABLE so that they can be extended in the future
#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "mutable", nested)]
pub struct CompleteExtendedType {
    // Empty. Available for future extension
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested, switch(u8))]
pub enum CompleteTypeObject {
    #[dust_dds(case = TK_ALIAS)]
    TkAlias { alias_type: CompleteAliasType },
    #[dust_dds(case = TK_ANNOTATION)]
    TkAnnotation {
        annotation_type: CompleteAnnotationType,
    },
    #[dust_dds(case = TK_STRUCTURE)]
    TkStructure { struct_type: CompleteStructType },
    #[dust_dds(case = TK_UNION)]
    TkUnion { union_type: CompleteUnionType },
    #[dust_dds(case = TK_BITSET)]
    TkBitset { bitset_type: CompleteBitsetType },
    #[dust_dds(case = TK_SEQUENCE)]
    TkSequence { sequence_type: CompleteSequenceType },
    #[dust_dds(case = TK_ARRAY)]
    TkArray { array_type: CompleteArrayType },
    #[dust_dds(case = TK_MAP)]
    TkMap { map_type: CompleteMapType },
    #[dust_dds(case = TK_ENUM)]
    TkEnum {
        enumerated_type: CompleteEnumeratedType,
    },
    #[dust_dds(case = TK_BITMASK)]
    TkBitmask { bitmask_type: CompleteBitmaskType },
    // =================== Future extensibility ============
    #[dust_dds(default)]
    Default { extended_type: CompleteExtendedType },
}

#[derive(DdsType, Debug, Clone, PartialEq, Default)]
#[dust_dds(extensibility = "mutable", nested)]
pub struct MinimalExtendedType {
    // Empty. Available for future extension
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested, switch(u8))]
pub enum MinimalTypeObject {
    #[dust_dds(case = TK_ALIAS)]
    TkAlias { alias_type: MinimalAliasType },
    #[dust_dds(case = TK_ANNOTATION)]
    TkAnnotation {
        annotation_type: MinimalAnnotationType,
    },
    #[dust_dds(case = TK_STRUCTURE)]
    TkStructure { struct_type: MinimalStructType },
    #[dust_dds(case = TK_UNION)]
    TkUnion { union_type: MinimalUnionType },
    #[dust_dds(case = TK_BITSET)]
    TkBitset { bitset_type: MinimalBitsetType },
    #[dust_dds(case = TK_SEQUENCE)]
    TkSequence { sequence_type: MinimalSequenceType },
    #[dust_dds(case = TK_ARRAY)]
    TkArray { array_type: MinimalArrayType },
    #[dust_dds(case = TK_MAP)]
    TkMap { map_type: MinimalMapType },
    #[dust_dds(case = TK_ENUM)]
    TkEnum {
        enumerated_type: MinimalEnumeratedType,
    },
    #[dust_dds(case = TK_BITMASK)]
    TkBitmask { bitmask_type: MinimalBitmaskType },
    // =================== Future extensibility ============
    #[dust_dds(default)]
    Default { extended_type: MinimalExtendedType },
}

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "appendable", nested, switch(u8))]
#[allow(clippy::large_enum_variant)]
pub enum TypeObject {
    #[dust_dds(case=EK_COMPLETE)]
    EkComplete { complete: CompleteTypeObject },
    #[dust_dds(case=EK_MINIMAL)]
    EkMinimal { minimal: MinimalTypeObject },
}
pub type TypeObjectSeq = Vec<TypeObject>;
// Set of TypeObjects representing a strong component: Equivalence class
// for the Strong Connectivity relationship (mutual reachability between
// types).
// Ordered by fully qualified typename lexicographic order
pub type StronglyConnectedComponent = TypeObjectSeq;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct TypeIdentifierTypeObjectPair {
    pub type_identifier: TypeIdentifier,
    pub type_object: TypeObject,
}
pub type TypeIdentifierTypeObjectPairSeq = Vec<TypeIdentifierTypeObjectPair>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "final", nested)]
pub struct TypeIdentifierPair {
    pub type_identifier1: TypeIdentifier,
    pub type_identifier2: TypeIdentifier,
}
pub type TypeIdentifierPairSeq = Vec<TypeIdentifierPair>;

#[derive(DdsType, Debug, Clone, PartialEq, Default)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct TypeIdentifierWithSize {
    pub type_id: TypeIdentifier,
    pub typeobject_serialized_size: u32,
}
pub type TypeIdentfierWithSizeSeq = Vec<TypeIdentifierWithSize>;

#[derive(DdsType, Debug, Clone, PartialEq, Default)]
#[dust_dds(extensibility = "appendable", nested)]
pub struct TypeIdentifierWithDependencies {
    pub typeid_with_size: TypeIdentifierWithSize,
    // The total additional types related to minimal_type
    pub dependent_typeid_count: i32,
    pub dependent_typeids: Vec<TypeIdentifierWithSize>,
}
pub type TypeIdentifierWithDependenciesSeq = Vec<TypeIdentifierWithDependencies>;

#[derive(DdsType, Debug, Clone, PartialEq)]
#[dust_dds(extensibility = "mutable", nested)]
pub struct TypeInformation {
    #[dust_dds(id = 0x1001)]
    pub minimal: TypeIdentifierWithDependencies,
    #[dust_dds(id = 0x1002)]
    pub complete: TypeIdentifierWithDependencies,
}
pub type TypeInformationSeq = Vec<TypeInformation>;

impl From<&DynamicType> for MinimalTypeObject {
    fn from(value: &DynamicType) -> Self {
        match value.get_kind() {
            TypeKind::STRUCTURE => {
                let mut struct_flags = match value.descriptor.extensibility_kind {
                    ExtensibilityKind::Final => TYPE_FLAG_IS_FINAL,
                    ExtensibilityKind::Appendable => TYPE_FLAG_IS_APPENDABLE,
                    ExtensibilityKind::Mutable => TYPE_FLAG_IS_MUTABLE,
                };
                if value.descriptor.is_nested {
                    struct_flags |= TYPE_FLAG_IS_NESTED
                };

                let header = MinimalStructHeader {
                    base_type: TypeIdentifier::TkNone, // TODO: Include base type
                    detail: MinimalTypeDetail {},
                };
                let member_seq = value.member_list.iter().map(From::from).collect();
                let struct_type = MinimalStructType {
                    struct_flags,
                    header,
                    member_seq,
                };
                MinimalTypeObject::TkStructure { struct_type }
            }
            t => todo!("Not yet implemeneted for {t:?}"),
        }
    }
}

impl From<&DynamicType> for CompleteTypeObject {
    fn from(value: &DynamicType) -> Self {
        match value.get_kind() {
            TypeKind::STRUCTURE => {
                let mut struct_flags = match value.descriptor.extensibility_kind {
                    ExtensibilityKind::Final => TYPE_FLAG_IS_FINAL,
                    ExtensibilityKind::Appendable => TYPE_FLAG_IS_APPENDABLE,
                    ExtensibilityKind::Mutable => TYPE_FLAG_IS_MUTABLE,
                };
                if value.descriptor.is_nested {
                    struct_flags |= TYPE_FLAG_IS_NESTED
                };

                let header = CompleteStructHeader {
                    base_type: TypeIdentifier::TkNone, // TODO: Include base type
                    detail: CompleteTypeDetail {
                        type_name: String::from(value.get_name()),
                        // TODO: Implement annotations
                        ann_builtin: None,
                        ann_custom: None,
                    },
                };
                let member_seq = value.member_list.iter().map(From::from).collect();
                let struct_type = CompleteStructType {
                    struct_flags,
                    header,
                    member_seq,
                };
                CompleteTypeObject::TkStructure { struct_type }
            }
            t => todo!("Not yet implemeneted for {t:?}"),
        }
    }
}

impl From<&DynamicType> for TypeIdentifier {
    fn from(value: &DynamicType) -> Self {
        match value.descriptor.kind {
            TypeKind::NONE => TypeIdentifier::TkNone,
            TypeKind::BOOLEAN => TypeIdentifier::TkBoolean,
            TypeKind::BYTE => TypeIdentifier::TkByteType,
            TypeKind::INT16 => TypeIdentifier::TkInt16Type,
            TypeKind::INT32 => TypeIdentifier::TkInt32Type,
            TypeKind::INT64 => TypeIdentifier::TkInt64Type,
            TypeKind::UINT16 => TypeIdentifier::TkUint16Type,
            TypeKind::UINT32 => TypeIdentifier::TkUint32Type,
            TypeKind::UINT64 => TypeIdentifier::TkUint64Type,
            TypeKind::FLOAT32 => TypeIdentifier::TkFloat32Type,
            TypeKind::FLOAT64 => TypeIdentifier::TkFloat64Type,
            TypeKind::FLOAT128 => TypeIdentifier::TkFloat128Type,
            TypeKind::INT8 => TypeIdentifier::TkInt8Type,
            TypeKind::UINT8 => TypeIdentifier::TkUint8Type,
            TypeKind::CHAR8 => TypeIdentifier::TkChar8Type,
            TypeKind::CHAR16 => TypeIdentifier::TkChar16Type,
            TypeKind::STRING8 => TypeIdentifier::TiString8Large {
                string_ldefn: StringLTypeDefn { bound: u32::MAX },
            },
            TypeKind::STRING16 => TypeIdentifier::TiString16Small {
                string_sdefn: StringSTypeDefn { bound: u8::MAX },
            },
            TypeKind::ALIAS => todo!(),
            TypeKind::ENUM => todo!(),
            TypeKind::BITMASK => todo!(),
            TypeKind::ANNOTATION => todo!(),
            TypeKind::STRUCTURE => todo!(),
            TypeKind::UNION => todo!(),
            TypeKind::BITSET => todo!(),
            TypeKind::SEQUENCE => todo!(),
            TypeKind::ARRAY => todo!(),
            TypeKind::MAP => todo!(),
        }
    }
}

impl From<&DynamicTypeMember> for CommonStructMember {
    fn from(value: &DynamicTypeMember) -> Self {
        let mut member_flags = match value.descriptor.try_construct_kind {
            TryConstructKind::UseDefault => MEMBER_FLAG_TRY_CONSTRUCT2,
            TryConstructKind::Discard => MEMBER_FLAG_TRY_CONSTRUCT1,
            TryConstructKind::Trim => MEMBER_FLAG_TRY_CONSTRUCT1 | MEMBER_FLAG_TRY_CONSTRUCT2,
        };
        if value.descriptor.is_key {
            member_flags |= MEMBER_FLAG_IS_KEY;
        }
        if value.descriptor.is_must_understand {
            member_flags |= MEMBER_FLAG_IS_MUST_UNDERSTAND;
        }
        if value.descriptor.is_optional {
            member_flags |= MEMBER_FLAG_IS_OPTIONAL;
        }

        CommonStructMember {
            member_id: value.get_id(),
            member_flags,
            member_type_id: (&value.descriptor.r#type).into(),
        }
    }
}

impl From<&DynamicTypeMember> for MinimalStructMember {
    fn from(value: &DynamicTypeMember) -> Self {
        let common = value.into();
        let name_hash = <[u8; 16]>::from(md5::compute(value.get_name().as_bytes()));
        let detail = MinimalMemberDetail {
            name_hash: [name_hash[0], name_hash[1], name_hash[2], name_hash[3]],
        };
        MinimalStructMember { common, detail }
    }
}

impl From<&DynamicTypeMember> for CompleteStructMember {
    fn from(value: &DynamicTypeMember) -> Self {
        let common = value.into();
        let detail = CompleteMemberDetail {
            name: String::from(value.get_name()),
            // TODO: Applied builtin and custom annotations
            ann_builtin: None,
            ann_custom: None,
        };
        CompleteStructMember { common, detail }
    }
}

impl TryFrom<&DynamicType> for TypeInformation {
    type Error = XTypesError;

    fn try_from(value: &DynamicType) -> Result<Self, Self::Error> {
        let minimal_type_object = TypeObject::EkMinimal {
            minimal: MinimalTypeObject::from(value),
        };
        let mut data = DynamicDataFactory::create_data(TypeObject::get_type());
        minimal_type_object.create_dynamic_sample(&mut data);
        let serialized_minimal_type_object = serialize_cdr2_le(&data)?;
        let hash_minimal_type_object = md5::compute(&serialized_minimal_type_object);

        let complete_type_object = TypeObject::EkComplete {
            complete: CompleteTypeObject::from(value),
        };
        let mut data = DynamicDataFactory::create_data(TypeObject::get_type());
        complete_type_object.create_dynamic_sample(&mut data);
        let serialized_complete_type_object = serialize_cdr2_le(&data)?;
        let hash_complete_type_object = md5::compute(&serialized_complete_type_object);

        Ok(TypeInformation {
            minimal: TypeIdentifierWithDependencies {
                typeid_with_size: TypeIdentifierWithSize {
                    type_id: TypeIdentifier::EkMinimal {
                        equivalence_hash: [
                            hash_minimal_type_object[0],
                            hash_minimal_type_object[1],
                            hash_minimal_type_object[2],
                            hash_minimal_type_object[3],
                            hash_minimal_type_object[4],
                            hash_minimal_type_object[5],
                            hash_minimal_type_object[6],
                            hash_minimal_type_object[7],
                            hash_minimal_type_object[8],
                            hash_minimal_type_object[9],
                            hash_minimal_type_object[10],
                            hash_minimal_type_object[11],
                            hash_minimal_type_object[12],
                            hash_minimal_type_object[13],
                        ],
                    },
                    typeobject_serialized_size: serialized_minimal_type_object.len() as u32,
                },
                dependent_typeid_count: -1,
                dependent_typeids: Vec::new(),
            },
            complete: TypeIdentifierWithDependencies {
                typeid_with_size: TypeIdentifierWithSize {
                    type_id: TypeIdentifier::EkComplete {
                        equivalence_hash: [
                            hash_complete_type_object[0],
                            hash_complete_type_object[1],
                            hash_complete_type_object[2],
                            hash_complete_type_object[3],
                            hash_complete_type_object[4],
                            hash_complete_type_object[5],
                            hash_complete_type_object[6],
                            hash_complete_type_object[7],
                            hash_complete_type_object[8],
                            hash_complete_type_object[9],
                            hash_complete_type_object[10],
                            hash_complete_type_object[11],
                            hash_complete_type_object[12],
                            hash_complete_type_object[13],
                        ],
                    },
                    typeobject_serialized_size: serialized_complete_type_object.len() as u32,
                },
                dependent_typeid_count: -1,
                dependent_typeids: Vec::new(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn shape_type_hash() {
        #[derive(Debug, PartialEq, TypeSupport)]
        #[dust_dds(extensibility = "final")]
        struct ShapeType {
            #[dust_dds(key)]
            color: String,
            x: i32,
            y: i32,
            shapesize: i32,
        }

        let type_information = TypeInformation::try_from(&ShapeType::get_type()).unwrap();
        assert_eq!(
            type_information
                .complete
                .typeid_with_size
                .typeobject_serialized_size,
            132
        );
        // assert_eq!(
        //     type_information.complete.typeid_with_size.type_id,
        //     TypeIdentifier::EkComplete {
        //         equivalence_hash: [
        //             0xce, 0x6d, 0x79, 0x13, 0x05, 0x8d, 0xaa, 0x30, 0x78, 0xa8, 0x8f, 0x98, 0x21,
        //             0x96
        //         ]
        //     }
        // );

        assert_eq!(
            type_information
                .minimal
                .typeid_with_size
                .typeobject_serialized_size,
            92
        );
        // assert_eq!(
        //     type_information.minimal.typeid_with_size.type_id,
        //     TypeIdentifier::EkMinimal {
        //         equivalence_hash: [
        //             0xd3, 0xd5, 0x89, 0xfb, 0x12, 0x8b, 0x55, 0xdb, 0x4b, 0x83, 0x3d, 0x99, 0xa4,
        //             0x02
        //         ]
        //     }
        // );
    }
}
