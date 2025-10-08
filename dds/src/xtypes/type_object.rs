use alloc::{boxed::Box, string::String, vec::Vec};

pub trait XTypesTypeObject {
    fn type_object() -> TypeObject;
}

use crate::xtypes::dynamic_type::TypeKind;

use super::dynamic_type::{DynamicType, TryConstructKind};

/* Manually created from dds-xtypes_typeobject.idl */

// ---------- Equivalence Kinds -------------------
pub type EquivalenceKind = u8;
pub const EK_MINIMAL: EquivalenceKind = 0xF1; // 0x1111 0001
pub const EK_COMPLETE: EquivalenceKind = 0xF2; // 0x1111 0010
pub const EK_BOTH: EquivalenceKind = 0xF3; // 0x1111 0011

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

// @extensibility(FINAL) @nested
#[repr(u8)]
pub enum TypeObjectHashId {
    EkComplete { hash: EquivalenceHash },
    EkMinimal { hash: EquivalenceHash },
}

// Flags that apply to struct/union/collection/enum/bitmask/bitset
// members/elements and DO affect type assignability
// Depending on the flag it may not apply to members of all types
// When not all, the applicable member types are listed
// @bit_bound(16)
pub struct MemberFlag(pub u16);
// @position(0) TRY_CONSTRUCT1, // T1 | 00 = INVALID, 01 = DISCARD
// @position(1) TRY_CONSTRUCT2, // T2 | 10 = USE_DEFAULT, 11 = TRIM
// @position(2) IS_EXTERNAL, // X StructMember, UnionMember,
// // CollectionElement
// @position(3) IS_OPTIONAL, // O StructMember
// @position(4) IS_MUST_UNDERSTAND, // M StructMember
// @position(5) IS_KEY, // K StructMember, UnionDiscriminator
// @position(6) IS_DEFAULT // D UnionMember, EnumerationLiteral

pub struct CollectionElementFlag {
    pub try_construct: TryConstructKind,
    pub is_external: bool,
} // T1, T2, X

pub struct StructMemberFlag {
    pub try_construct: TryConstructKind,
    pub is_external: bool,
    pub is_optional: bool,
    pub is_must_undestand: bool,
    pub is_key: bool,
} // T1, T2, O, M, K, X
pub struct UnionMemberFlag {
    pub try_construct: TryConstructKind,
    pub is_default: bool,
    pub is_external: bool,
} // T1, T2, D, X
pub struct UnionDiscriminatorFlag {
    pub try_construct: TryConstructKind,
    pub is_key: bool,
} // T1, T2, K
pub struct EnumeratedLiteralFlag {
    pub is_default: bool,
} // D
pub struct AnnotationParameterFlag; // Unused. No flags apply
pub struct AliasMemberFlag; // Unused. No flags apply
pub struct BitflagFlag; // Unused. No flags apply
pub struct BitsetMemberFlag; // Unused. No flags apply

// Mask used to remove the flags that do no affect assignability
// Selects T1, T2, O, M, K, D
pub const MEMBER_FLAG_MINIMAL_MASK: u16 = 0x003f;
// Flags that apply to type declarationa and DO affect assignability
// Depending on the flag it may not apply to all types
// When not all, the applicable types are listed
// @bit_bound(16)
pub struct TypeFlag(pub u16);
// @position(0) IS_FINAL, // F |
// @position(1) IS_APPENDABLE, // A |- Struct, Union
// @position(2) IS_MUTABLE, // M | (exactly one flag)
// @position(3) IS_NESTED, // N Struct, Union
// @position(4) IS_AUTOID_HASH // H Struct

//@bit_bound(16)
pub struct StructTypeFlag {
    pub is_final: bool,
    pub is_appendable: bool,
    pub is_mutable: bool,
    pub is_nested: bool,
    pub is_autoid_hash: bool,
}

pub struct UnionTypeFlag {
    pub is_final: bool,
    pub is_appendable: bool,
    pub is_mutable: bool,
    pub is_nested: bool,
    pub is_autoid_hash: bool,
} // All flags apply
pub struct CollectionTypeFlag; // Unused. No flags apply
pub struct AnnotationTypeFlag; // Unused. No flags apply
pub struct AliasTypeFlag; // Unused. No flags apply
pub struct EnumTypeFlag; // Unused. No flags apply
pub struct BitmaskTypeFlag; // Unused. No flags apply
pub struct BitsetTypeFlag; // Unused. No flags apply

// Mask used to remove the flags that do no affect assignability
pub const TYPE_FLAG_MINIMAL_MASK: u16 = 0x0007; // Selects M, A, F

// 1 Byte
// @extensibility(FINAL) @nested
pub struct StringSTypeDefn {
    pub bound: SBound,
}
// 4 Bytes
// @extensibility(FINAL) @nested
pub struct StringLTypeDefn {
    pub bound: LBound,
}

// @extensibility(FINAL) @nested
pub struct PlainCollectionHeader {
    pub equiv_kind: EquivalenceKind,
    pub element_flags: CollectionElementFlag,
}

// @extensibility(FINAL) @nested
pub struct PlainSequenceSElemDefn {
    pub header: PlainCollectionHeader,
    pub bound: SBound,
    pub element_identifier: TypeIdentifier,
}

// @extensibility(FINAL) @nested
pub struct PlainSequenceLElemDefn {
    pub header: PlainCollectionHeader,
    pub bound: LBound,
    pub element_identifier: TypeIdentifier,
}

// @extensibility(FINAL) @nested
pub struct PlainArraySElemDefn {
    pub header: PlainCollectionHeader,
    pub array_bound_seq: SBoundSeq,
    pub element_identifier: TypeIdentifier,
}

// @extensibility(FINAL) @nested
pub struct PlainArrayLElemDefn {
    pub header: PlainCollectionHeader,
    pub array_bound_seq: LBoundSeq,
    pub element_identifier: TypeIdentifier,
}

// @extensibility(FINAL) @nested
pub struct PlainMapSTypeDefn {
    pub header: PlainCollectionHeader,
    pub bound: SBound,
    pub element_identifier: TypeIdentifier,
    pub key_flags: CollectionElementFlag,
    pub key_identifier: TypeIdentifier,
}

// @extensibility(FINAL) @nested
pub struct PlainMapLTypeDefn {
    pub header: PlainCollectionHeader,
    pub bound: LBound,
    pub element_identifier: TypeIdentifier,
    pub key_flags: CollectionElementFlag,
    pub key_identifier: TypeIdentifier,
}

// Used for Types that have cyclic depencencies with other types
// @extensibility(APPENDABLE) @nested
pub struct StronglyConnectedComponentId {
    pub sc_component_id: TypeObjectHashId, // Hash StronglyConnectedComponent
    pub scc_length: i32,                   // StronglyConnectedComponent.length
    pub scc_index: i32,                    // identify type in Strongly Connected Comp.
}
// Future extensibility
// @extensibility(MUTABLE) @nested
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
// @extensibility(FINAL) @nested
#[repr(u8)]
pub enum TypeIdentifier {
    TkNone,
    TkBoolean,
    TkByteType,
    TkInt8Type,
    TkInt16Type,
    TkInt32Type,
    TkInt64Type,
    TkUint8Type,
    TkUint16Type,
    TkUint32Type,
    TkUint64Type,
    TkFloat32Type,
    TkFloat64Type,
    TkFloat128Type,
    TkChar8Type,
    TkChar16Type,
    // ============ Strings - use TypeIdentifierKind ===================
    TiString8Small {
        string_sdefn: StringSTypeDefn,
    },
    TiString16Small {
        string_sdefn: StringSTypeDefn,
    },
    TiString8Large {
        string_ldefn: StringLTypeDefn,
    },
    TiString16Large {
        string_ldefn: StringLTypeDefn,
    },
    // ============ Plain collectios - use TypeIdentifierKind =========
    TiPlainSequenceSmall {
        seq_sdefn: Box<PlainSequenceSElemDefn>,
    },
    TiPlainSequenceLarge {
        seq_ldefn: Box<PlainSequenceLElemDefn>,
    },
    TiPlainArraySmall {
        array_sdefn: Box<PlainArraySElemDefn>,
    },
    TiPlainArrayLarge {
        array_ldefn: Box<PlainArrayLElemDefn>,
    },
    TiPlainMapSmall {
        map_sdefn: Box<PlainMapSTypeDefn>,
    },
    TiPlainMapLarge {
        map_ldefn: Box<PlainMapLTypeDefn>,
    },
    // ============ Types that are mutually dependent on each other ===
    TiStronglyConnectedComponent {
        sc_component_id: StronglyConnectedComponentId,
    },
    // ============ The remaining cases - use EquivalenceKind =========
    EkComplete {
        // equivalence_hash: EquivalenceHash, // Original in IDL
        complete: Box<DynamicType>,
    },
    EkMinimal {
        minimal: Box<MinimalTypeObject>,
        // equivalence_hash: EquivalenceHash, // Original in IDL
    },
    // =================== Future extensibility ============
    // default:
    // MinimalExtendedType extended_type;
}
pub type TypeIdentifierSeq = Vec<TypeIdentifier>;

// --- Annotation usage: -----------------------------------------------
// ID of a type member
pub type MemberId = u32;
pub const ANNOTATION_STR_VALUE_MAX_LEN: u32 = 128;
pub const ANNOTATION_OCTETSEC_VALUE_MAX_LEN: u32 = 128;
// @extensibility(MUTABLE) @nested
pub struct ExtendedAnnotationParameterValue {
    // Empty. Available for future extension
}

/* Literal value of an annotation member: either the default value in its
* definition or the value applied in its usage.
*/
// @extensibility(FINAL) @nested
#[repr(u8)]
pub enum AnnotationParameterValue {
    TkBoolean {
        boolean_value: bool,
    },
    TkByte {
        byte_value: u8,
    },
    TkInt8 {
        int8_value: i8,
    },
    TkUint8 {
        uint8_value: u8,
    },
    TkInt16 {
        int16_value: i16,
    },
    TkUint16 {
        uint_16_value: u16,
    },
    TkInt32 {
        int32_value: i32,
    },
    TkUint32 {
        uint32_value: u32,
    },
    TkInt64 {
        int64_value: i64,
    },
    TkUint64 {
        uint64_value: u64,
    },
    TkFloat32 {
        float32_value: f32,
    },
    TkFloat64 {
        float64_value: f64,
    },
    // TypeKind::FLOAT128{
    // float128_value: f128},
    TkChar8 {
        char_value: char,
    },
    // TypeKind::CHAR16 {
    // wchar_value: char16},
    TkEnum {
        enumerated_value: i32,
    },
    TkString8 {
        string8_value: String, /*string<ANNOTATION_STR_VALUE_MAX_LEN>  */
    },
    // TypeKind::STRING16:
    // wstring<ANNOTATION_STR_VALUE_MAX_LEN> string16_value;
    // default:
    // ExtendedAnnotationParameterValue extended_value;
}

// The application of an annotation to some type or type member
// @extensibility(APPENDABLE) @nested
pub struct AppliedAnnotationParameter {
    pub paramname_hash: NameHash,
    pub value: AnnotationParameterValue,
}
// Sorted by AppliedAnnotationParameter.paramname_hash
pub type AppliedAnnotationParameterSeq = Vec<AppliedAnnotationParameter>;

// @extensibility(APPENDABLE) @nested
pub struct AppliedAnnotation {
    pub annotation_typeid: TypeIdentifier,
    pub param_seq: Option<AppliedAnnotationParameterSeq>,
}
// Sorted by AppliedAnnotation.annotation_typeid
pub type AppliedAnnotationSeq = Vec<AppliedAnnotation>;
// @verbatim(placement="<placement>", language="<lang>", text="<text>")
// @extensibility(FINAL) @nested
pub struct AppliedVerbatimAnnotation {
    pub placement: String, //string<32>
    pub language: String,  //string<32>
    pub text: String,
}

// --- Aggregate types: ------------------------------------------------
// @extensibility(APPENDABLE) @nested
pub struct AppliedBuiltinMemberAnnotations {
    pub unit: Option<String>,                  // @unit("<unit>")
    pub min: Option<AnnotationParameterValue>, // @min , @range
    pub max: Option<AnnotationParameterValue>, // @max , @range
    pub hash_id: Option<String>,               // @hashid("<membername>")
}

// @extensibility(FINAL) @nested
pub struct CommonStructMember {
    pub member_id: MemberId,
    pub member_flags: StructMemberFlag,
    pub member_type_id: TypeIdentifier,
}

// COMPLETE Details for a member of an aggregate type
// @extensibility(FINAL) @nested
pub struct CompleteMemberDetail {
    pub name: MemberName,
    pub ann_builtin: Option<AppliedBuiltinMemberAnnotations>,
    pub ann_custom: Option<AppliedAnnotationSeq>,
}
// MINIMAL Details for a member of an aggregate type
// @extensibility(FINAL) @nested
pub struct MinimalMemberDetail {
    pub name_hash: NameHash,
}

// Member of an aggregate type
// @extensibility(APPENDABLE) @nested
pub struct CompleteStructMember {
    pub common: CommonStructMember,
    pub detail: CompleteMemberDetail,
}
// Ordered by the member_index
pub type CompleteStructMemberSeq = Vec<CompleteStructMember>;
// Member of an aggregate type
// @extensibility(APPENDABLE) @nested
pub struct MinimalStructMember {
    pub common: CommonStructMember,
    pub detail: MinimalMemberDetail,
}
// Ordered by common.member_id
pub type MinimalStructMemberSeq = Vec<MinimalStructMember>;
// @extensibility(APPENDABLE) @nested
pub struct AppliedBuiltinTypeAnnotations {
    pub verbatim: Option<AppliedVerbatimAnnotation>, // @verbatim(...)
}

// @extensibility(FINAL) @nested
pub struct MinimalTypeDetail {
    // Empty. Available for future extension
}

// @extensibility(FINAL) @nested
pub struct CompleteTypeDetail {
    pub ann_builtin: Option<AppliedBuiltinTypeAnnotations>,
    pub ann_custom: Option<AppliedAnnotationSeq>,
    pub type_name: QualifiedTypeName,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteStructHeader {
    pub base_type: TypeIdentifier,
    pub detail: CompleteTypeDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalStructHeader {
    pub base_type: TypeIdentifier,
    pub detail: MinimalTypeDetail,
}

// @extensibility(FINAL) @nested
pub struct CompleteStructType {
    pub struct_flags: StructTypeFlag,
    pub header: CompleteStructHeader,
    pub member_seq: CompleteStructMemberSeq,
}

// @extensibility(FINAL) @nested
pub struct MinimalStructType {
    pub struct_flags: StructTypeFlag,
    pub header: MinimalStructHeader,
    pub member_seq: MinimalStructMemberSeq,
}

// --- Union: ----------------------------------------------------------
// labels that apply to a member of a union type
// Ordered by their values
pub type UnionCaseLabelSeq = Vec<i32>;
// @extensibility(FINAL) @nested
pub struct CommonUnionMember {
    pub member_id: MemberId,
    pub member_flags: UnionMemberFlag,
    pub type_id: TypeIdentifier,
    pub label_seq: UnionCaseLabelSeq,
}

// Member of a union type
// @extensibility(APPENDABLE) @nested
pub struct CompleteUnionMember {
    pub common: CommonUnionMember,
    pub detail: CompleteMemberDetail,
}
// Ordered by member_index
pub type CompleteUnionMemberSeq = Vec<CompleteUnionMember>;

// Member of a union type
// @extensibility(APPENDABLE) @nested
pub struct MinimalUnionMember {
    pub common: CommonUnionMember,
    pub detail: MinimalMemberDetail,
}
// Ordered by MinimalUnionMember.common.member_id
pub type MinimalUnionMemberSeq = Vec<MinimalUnionMember>;
// @extensibility(FINAL) @nested
pub struct CommonDiscriminatorMember {
    pub member_flags: UnionDiscriminatorFlag,
    pub type_id: TypeIdentifier,
}
// Member of a union type
// @extensibility(APPENDABLE) @nested
pub struct CompleteDiscriminatorMember {
    pub common: CommonDiscriminatorMember,
    pub ann_builtin: Option<AppliedBuiltinTypeAnnotations>,
    pub ann_custom: Option<AppliedAnnotationSeq>,
}
// Member of a union type
// @extensibility(APPENDABLE) @nested
pub struct MinimalDiscriminatorMember {
    pub common: CommonDiscriminatorMember,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteUnionHeader {
    pub detail: CompleteTypeDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalUnionHeader {
    pub detail: MinimalTypeDetail,
}

// @extensibility(FINAL) @nested
pub struct CompleteUnionType {
    pub union_flags: UnionTypeFlag,
    pub header: CompleteUnionHeader,
    pub discriminator: CompleteDiscriminatorMember,
    pub member_seq: CompleteUnionMemberSeq,
}

// @extensibility(FINAL) @nested
pub struct MinimalUnionType {
    pub union_flags: UnionTypeFlag,
    pub header: MinimalUnionHeader,
    pub discriminator: MinimalDiscriminatorMember,
    pub member_seq: MinimalUnionMemberSeq,
}

// --- Annotation: ----------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CommonAnnotationParameter {
    pub member_flags: AnnotationParameterFlag,
    pub member_type_id: TypeIdentifier,
}

// Member of an annotation type
// @extensibility(APPENDABLE) @nested
pub struct CompleteAnnotationParameter {
    pub common: CommonAnnotationParameter,
    pub name: MemberName,
    pub default_value: AnnotationParameterValue,
}
// Ordered by CompleteAnnotationParameter.name
pub type CompleteAnnotationParameterSeq = Vec<CompleteAnnotationParameter>;

// @extensibility(APPENDABLE) @nested
pub struct MinimalAnnotationParameter {
    pub common: CommonAnnotationParameter,
    pub name_hash: NameHash,
    pub default_value: AnnotationParameterValue,
}
// Ordered by MinimalAnnotationParameter.name_hash
pub type MinimalAnnotationParameterSeq = Vec<MinimalAnnotationParameter>;
// @extensibility(APPENDABLE) @nested
pub struct CompleteAnnotationHeader {
    pub annotation_name: QualifiedTypeName,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalAnnotationHeader {
    // Empty. Available for future extension
}

// @extensibility(FINAL) @nested
pub struct CompleteAnnotationType {
    pub annotation_flag: AnnotationTypeFlag,
    pub header: CompleteAnnotationHeader,
    pub member_seq: CompleteAnnotationParameterSeq,
}

// @extensibility(FINAL) @nested
pub struct MinimalAnnotationType {
    pub annotation_flag: AnnotationTypeFlag,
    pub header: MinimalAnnotationHeader,
    pub member_seq: MinimalAnnotationParameterSeq,
}

// --- Alias: ----------------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CommonAliasBody {
    pub related_flags: AliasMemberFlag,
    pub related_type: TypeIdentifier,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteAliasBody {
    pub common: CommonAliasBody,
    pub ann_builtin: Option<AppliedBuiltinMemberAnnotations>,
    pub ann_custom: Option<AppliedAnnotationSeq>,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalAliasBody {
    pub common: CommonAliasBody,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteAliasHeader {
    pub detail: CompleteTypeDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalAliasHeader {
    // Empty. Available for future extension
}

// @extensibility(FINAL) @nested
pub struct CompleteAliasType {
    pub alias_flags: AliasTypeFlag,
    pub header: CompleteAliasHeader,
    pub body: CompleteAliasBody,
}

// @extensibility(FINAL) @nested
pub struct MinimalAliasType {
    pub alias_flags: AliasTypeFlag,
    pub header: MinimalAliasHeader,
    pub body: MinimalAliasBody,
}

// --- Collections: ----------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CompleteElementDetail {
    pub ann_builtin: Option<AppliedBuiltinMemberAnnotations>,
    pub ann_custom: Option<AppliedAnnotationSeq>,
}

// @extensibility(FINAL) @nested
pub struct CommonCollectionElement {
    pub element_flags: CollectionElementFlag,
    pub _type: TypeIdentifier,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteCollectionElement {
    pub common: CommonCollectionElement,
    pub detail: CompleteElementDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalCollectionElement {
    pub common: CommonCollectionElement,
}

// @extensibility(FINAL) @nested
pub struct CommonCollectionHeader {
    pub bound: LBound,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteCollectionHeader {
    pub common: CommonCollectionHeader,
    pub detail: Option<CompleteTypeDetail>, // not present for anonymous
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalCollectionHeader {
    pub common: CommonCollectionHeader,
}

// --- Sequence: ------------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CompleteSequenceType {
    pub collection_flag: CollectionTypeFlag,
    pub header: CompleteCollectionHeader,
    pub element: CompleteCollectionElement,
}

// @extensibility(FINAL) @nested
pub struct MinimalSequenceType {
    pub collection_flag: CollectionTypeFlag,
    pub header: MinimalCollectionHeader,
    pub element: MinimalCollectionElement,
}

// --- Array: ------------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CommonArrayHeader {
    pub bound_seq: LBoundSeq,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteArrayHeader {
    pub common: CommonArrayHeader,
    pub detail: CompleteTypeDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalArrayHeader {
    pub common: CommonArrayHeader,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteArrayType {
    pub collection_flag: CollectionTypeFlag,
    pub header: CompleteArrayHeader,
    pub element: CompleteCollectionElement,
}

// @extensibility(FINAL) @nested
pub struct MinimalArrayType {
    pub collection_flag: CollectionTypeFlag,
    pub header: MinimalArrayHeader,
    pub element: MinimalCollectionElement,
}

// --- Map: ------------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CompleteMapType {
    pub collection_flag: CollectionTypeFlag,
    pub header: CompleteCollectionHeader,
    pub key: CompleteCollectionElement,
    pub element: CompleteCollectionElement,
}

// @extensibility(FINAL) @nested
pub struct MinimalMapType {
    pub collection_flag: CollectionTypeFlag,
    pub header: MinimalCollectionHeader,
    pub key: MinimalCollectionElement,
    pub element: MinimalCollectionElement,
}

// --- Enumeration: ----------------------------------------------------
pub type BitBound = u16;
// Constant in an enumerated type
// @extensibility(APPENDABLE) @nested
pub struct CommonEnumeratedLiteral {
    pub value: i32,
    pub flags: EnumeratedLiteralFlag,
}

// Constant in an enumerated type
// @extensibility(APPENDABLE) @nested
pub struct CompleteEnumeratedLiteral {
    pub common: CommonEnumeratedLiteral,
    pub detail: CompleteMemberDetail,
}

// Ordered by EnumeratedLiteral.common.value
pub type CompleteEnumeratedLiteralSeq = Vec<CompleteEnumeratedLiteral>;

// Constant in an enumerated type
// @extensibility(APPENDABLE) @nested
pub struct MinimalEnumeratedLiteral {
    pub common: CommonEnumeratedLiteral,
    pub detail: MinimalMemberDetail,
}

// Ordered by EnumeratedLiteral.common.value
pub type MinimalEnumeratedLiteralSeq = Vec<MinimalEnumeratedLiteral>;
// @extensibility(FINAL) @nested
pub struct CommonEnumeratedHeader {
    pub bit_bound: BitBound,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteEnumeratedHeader {
    pub common: CommonEnumeratedHeader,
    pub detail: CompleteTypeDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalEnumeratedHeader {
    pub common: CommonEnumeratedHeader,
}

// Enumerated type
// @extensibility(FINAL) @nested
pub struct CompleteEnumeratedType {
    pub enum_flags: EnumTypeFlag, // unused
    pub header: CompleteEnumeratedHeader,
    pub literal_seq: CompleteEnumeratedLiteralSeq,
}

// Enumerated type
// @extensibility(FINAL) @nested
pub struct MinimalEnumeratedType {
    pub enum_flags: EnumTypeFlag, // unused
    pub header: MinimalEnumeratedHeader,
    pub literal_seq: MinimalEnumeratedLiteralSeq,
}

// --- Bitmask: --------------------------------------------------------
// Bit in a bit mask
// @extensibility(FINAL) @nested
pub struct CommonBitflag {
    pub position: u16,
    pub flags: BitflagFlag,
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteBitflag {
    pub common: CommonBitflag,
    pub detail: CompleteMemberDetail,
}
// Ordered by Bitflag.position
pub type CompleteBitflagSeq = Vec<CompleteBitflag>;

// @extensibility(APPENDABLE) @nested
pub struct MinimalBitflag {
    pub common: CommonBitflag,
    pub detail: MinimalMemberDetail,
}
// Ordered by Bitflag.position
pub type MinimalBitflagSeq = Vec<MinimalBitflag>;
// @extensibility(FINAL) @nested
pub struct CommonBitmaskHeader {
    pub bit_bound: BitBound,
}
pub type CompleteBitmaskHeader = CompleteEnumeratedHeader;
pub type MinimalBitmaskHeader = MinimalEnumeratedHeader;
// @extensibility(APPENDABLE) @nested
pub struct CompleteBitmaskType {
    pub bitmask_flags: BitmaskTypeFlag, // unused
    pub header: CompleteBitmaskHeader,
    pub flag_seq: CompleteBitflagSeq,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalBitmaskType {
    pub bitmask_flags: BitmaskTypeFlag, // unused
    pub header: MinimalBitmaskHeader,
    pub flag_seq: MinimalBitflagSeq,
}

// --- Bitset: ----------------------------------------------------------
// @extensibility(FINAL) @nested
pub struct CommonBitfield {
    pub position: u16,
    pub flags: BitsetMemberFlag,
    pub bitcount: u8,
    pub holder_type: TypeKind, // Must be primitive integer type
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteBitfield {
    pub common: CommonBitfield,
    pub detail: CompleteMemberDetail,
}

// Ordered by Bitfield.position
pub type CompleteBitfieldSeq = Vec<CompleteBitfield>;

// @extensibility(APPENDABLE) @nested
pub struct MinimalBitfield {
    pub common: CommonBitfield,
    pub name_hash: NameHash,
}

// Ordered by Bitfield.position
pub type MinimalBitfieldSeq = Vec<MinimalBitfield>;
// @extensibility(APPENDABLE) @nested
pub struct CompleteBitsetHeader {
    pub detail: CompleteTypeDetail,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalBitsetHeader {
    // Empty. Available for future extension
}

// @extensibility(APPENDABLE) @nested
pub struct CompleteBitsetType {
    pub bitset_flags: BitsetTypeFlag, // unused
    pub header: CompleteBitsetHeader,
    pub field_seq: CompleteBitfieldSeq,
}

// @extensibility(APPENDABLE) @nested
pub struct MinimalBitsetType {
    pub bitset_flags: BitsetTypeFlag, // unused
    pub header: MinimalBitsetHeader,
    pub field_seq: MinimalBitfieldSeq,
}

// --- Type Object: ---------------------------------------------------
// The types associated with each selection must have extensibility
// kind APPENDABLE or MUTABLE so that they can be extended in the future
// @extensibility(MUTABLE) @nested
pub struct CompleteExtendedType {
    // Empty. Available for future extension
}

// @extensibility(FINAL) @nested
#[repr(u8)]
pub enum CompleteTypeObject {
    TkAlias {
        alias_type: CompleteAliasType,
    },
    TkAnnotation {
        annotation_type: CompleteAnnotationType,
    },
    TkStructure {
        struct_type: CompleteStructType,
    },
    TkUnion {
        union_type: CompleteUnionType,
    },
    TkBitset {
        bitset_type: CompleteBitsetType,
    },
    TkSequence {
        sequence_type: CompleteSequenceType,
    },
    TkArray {
        array_type: CompleteArrayType,
    },
    TkMap {
        map_type: CompleteMapType,
    },
    TkEnum {
        enumerated_type: CompleteEnumeratedType,
    },
    TkBitmask {
        bitmask_type: CompleteBitmaskType,
    }, // =================== Future extensibility ============
       // default:
       // CompleteExtendedType extended_type;
}

// @extensibility(MUTABLE) @nested
pub struct MinimalExtendedType {
    // Empty. Available for future extension
}

// @extensibility(FINAL) @nested
#[repr(u8)]
pub enum MinimalTypeObject {
    TkAlias {
        alias_type: MinimalAliasType,
    },
    TkAnnotation {
        annotation_type: MinimalAnnotationType,
    },
    TkStructure {
        struct_type: MinimalStructType,
    },
    TkUnion {
        union_type: MinimalUnionType,
    },
    TkBitset {
        bitset_type: MinimalBitsetType,
    },
    TkSequence {
        sequence_type: MinimalSequenceType,
    },
    TkArray {
        array_type: MinimalArrayType,
    },
    TkMap {
        map_type: MinimalMapType,
    },
    TkEnum {
        enumerated_type: MinimalEnumeratedType,
    },
    TkBitmask {
        bitmask_type: MinimalBitmaskType,
    },
    // =================== Future extensibility ============
    // default:
    // MinimalExtendedType extended_type;
}

// @extensibility(APPENDABLE) @nested
#[repr(u8)]
#[allow(clippy::large_enum_variant)]
pub enum TypeObject {
    // EquivalenceKind
    EkComplete { complete: CompleteTypeObject },
    EkMinimal { minimal: MinimalTypeObject },
}
pub type TypeObjectSeq = Vec<TypeObject>;
// Set of TypeObjects representing a strong component: Equivalence class
// for the Strong Connectivity relationship (mutual reachability between
// types).
// Ordered by fully qualified typename lexicographic order
pub type StronglyConnectedComponent = TypeObjectSeq;

pub struct TypeIdentifierTypeObjectPair {
    pub type_identifier: TypeIdentifier,
    pub type_object: TypeObject,
}
pub type TypeIdentifierTypeObjectPairSeq = Vec<TypeIdentifierTypeObjectPair>;

// @extensibility(FINAL) @nested
pub struct TypeIdentifierPair {
    pub type_identifier1: TypeIdentifier,
    pub type_identifier2: TypeIdentifier,
}
pub type TypeIdentifierPairSeq = Vec<TypeIdentifierPair>;

// @extensibility(APPENDABLE) @nested
pub struct TypeIdentifierWithSize {
    pub type_id: TypeIdentifier,
    pub typeobject_serialized_size: u32,
}
pub type TypeIdentfierWithSizeSeq = Vec<TypeIdentifierWithSize>;

// @extensibility(APPENDABLE) @nested
pub struct TypeIdentifierWithDependencies {
    pub typeid_with_size: TypeIdentifierWithSize,
    // The total additional types related to minimal_type
    pub dependent_typeid_count: i32,
    pub dependent_typeids: Vec<TypeIdentifierWithSize>,
}
pub type TypeIdentifierWithDependenciesSeq = Vec<TypeIdentifierWithDependencies>;

// @extensibility(MUTABLE) @nested
pub struct TypeInformation {
    pub minimal: TypeIdentifierWithDependencies, //@id(0x1001)
    pub complete: TypeIdentifierWithDependencies, //@id(0x1002)
}
pub type TypeInformationSeq = Vec<TypeInformation>;
