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
    // ============ Hash-only variants for wire deserialization =========
    // These are used when deserializing TypeInformation from the wire.
    // The full type must be looked up via TypeLookup.
    EkCompleteHash {
        hash: EquivalenceHash,
    },
    EkMinimalHash {
        hash: EquivalenceHash,
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

// ============================================================================
// TypeObject Serialization for TypeLookup
// ============================================================================

impl TypeObject {
    /// Serializes this TypeObject to XCDR2 bytes for TypeLookup response.
    ///
    /// The format follows XTypes 1.3 specification:
    /// - TypeObject is APPENDABLE, so it has a DHEADER (4 bytes length)
    /// - Discriminator (1 byte): 0xF2 for EkComplete, 0xF1 for EkMinimal
    /// - Payload: CompleteTypeObject or MinimalTypeObject
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        match self {
            TypeObject::EkComplete { complete } => {
                // Discriminator for EkComplete
                buffer.push(EK_COMPLETE);

                // Serialize CompleteTypeObject
                complete.serialize_to(&mut buffer);
            }
            TypeObject::EkMinimal { .. } => {
                // Discriminator for EkMinimal
                buffer.push(EK_MINIMAL);
                // MinimalTypeObject serialization not implemented yet
            }
        }

        // For APPENDABLE types, we need DHEADER. Prepend it.
        let mut result = Vec::with_capacity(4 + buffer.len());
        result.extend_from_slice(&(buffer.len() as u32).to_le_bytes());
        result.extend_from_slice(&buffer);
        result
    }

    /// Deserializes a TypeObject from XCDR2 bytes.
    ///
    /// Returns the TypeObject and the number of bytes consumed.
    pub fn deserialize_from_bytes(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // Read DHEADER (4 bytes length for APPENDABLE type)
        if data.len() < 4 {
            return None;
        }
        let dheader = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        cursor += 4;

        if data.len() < cursor + dheader {
            return None;
        }

        // Read discriminator
        let discriminator = data[cursor];
        cursor += 1;

        match discriminator {
            EK_COMPLETE => {
                let (complete, bytes_read) = CompleteTypeObject::deserialize_from(&data[cursor..])?;
                cursor += bytes_read;
                Some((TypeObject::EkComplete { complete }, cursor))
            }
            EK_MINIMAL => {
                // MinimalTypeObject deserialization not implemented
                None
            }
            _ => None,
        }
    }
}

impl CompleteTypeObject {
    /// Serializes this CompleteTypeObject to bytes.
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        match self {
            CompleteTypeObject::TkStructure { struct_type } => {
                // Discriminator for TkStructure (TypeKind::STRUCTURE = 0x51)
                buffer.push(TypeKind::STRUCTURE as u8);
                struct_type.serialize_to(buffer);
            }
            // Other type kinds - serialize discriminator only for now
            CompleteTypeObject::TkAlias { .. } => buffer.push(TypeKind::ALIAS as u8),
            CompleteTypeObject::TkEnum { .. } => buffer.push(TypeKind::ENUM as u8),
            CompleteTypeObject::TkUnion { .. } => buffer.push(TypeKind::UNION as u8),
            CompleteTypeObject::TkSequence { .. } => buffer.push(TypeKind::SEQUENCE as u8),
            CompleteTypeObject::TkArray { .. } => buffer.push(TypeKind::ARRAY as u8),
            CompleteTypeObject::TkMap { .. } => buffer.push(TypeKind::MAP as u8),
            CompleteTypeObject::TkBitmask { .. } => buffer.push(TypeKind::BITMASK as u8),
            CompleteTypeObject::TkBitset { .. } => buffer.push(TypeKind::BITSET as u8),
            CompleteTypeObject::TkAnnotation { .. } => buffer.push(TypeKind::ANNOTATION as u8),
        }
    }

    /// Deserializes a CompleteTypeObject from bytes.
    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        if data.is_empty() {
            return None;
        }

        let discriminator = data[0];
        let mut cursor = 1;

        match discriminator {
            x if x == TypeKind::STRUCTURE as u8 => {
                let (struct_type, bytes_read) = CompleteStructType::deserialize_from(&data[cursor..])?;
                cursor += bytes_read;
                Some((CompleteTypeObject::TkStructure { struct_type }, cursor))
            }
            // Other type kinds not yet supported for deserialization
            _ => None,
        }
    }
}

impl CompleteStructType {
    /// Serializes this CompleteStructType to bytes.
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // StructTypeFlag (2 bytes, u16)
        let flags = self.struct_flags.to_u16();
        buffer.extend_from_slice(&flags.to_le_bytes());

        // CompleteStructHeader (APPENDABLE - needs DHEADER)
        self.header.serialize_to(buffer);

        // member_seq (sequence length + members)
        buffer.extend_from_slice(&(self.member_seq.len() as u32).to_le_bytes());
        for member in &self.member_seq {
            member.serialize_to(buffer);
        }
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // StructTypeFlag (2 bytes)
        if data.len() < cursor + 2 {
            return None;
        }
        let flags_u16 = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
        let struct_flags = StructTypeFlag::from_u16(flags_u16);
        cursor += 2;

        // CompleteStructHeader (APPENDABLE - has DHEADER)
        let (header, bytes_read) = CompleteStructHeader::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        // member_seq (sequence length + members)
        if data.len() < cursor + 4 {
            return None;
        }
        let member_count = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        let mut member_seq = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            let (member, bytes_read) = CompleteStructMember::deserialize_from(&data[cursor..])?;
            cursor += bytes_read;
            member_seq.push(member);
        }

        Some((
            CompleteStructType {
                struct_flags,
                header,
                member_seq,
            },
            cursor,
        ))
    }
}

impl StructTypeFlag {
    fn to_u16(&self) -> u16 {
        let mut flags = 0u16;
        if self.is_final {
            flags |= 0x0001;
        }
        if self.is_appendable {
            flags |= 0x0002;
        }
        if self.is_mutable {
            flags |= 0x0004;
        }
        if self.is_nested {
            flags |= 0x0008;
        }
        if self.is_autoid_hash {
            flags |= 0x0010;
        }
        flags
    }

    fn from_u16(flags: u16) -> Self {
        StructTypeFlag {
            is_final: (flags & 0x0001) != 0,
            is_appendable: (flags & 0x0002) != 0,
            is_mutable: (flags & 0x0004) != 0,
            is_nested: (flags & 0x0008) != 0,
            is_autoid_hash: (flags & 0x0010) != 0,
        }
    }
}

impl CompleteStructHeader {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // APPENDABLE type needs DHEADER
        let mut content = Vec::new();

        // base_type: TypeIdentifier
        self.base_type.serialize_to(&mut content);

        // detail: CompleteTypeDetail
        self.detail.serialize_to(&mut content);

        // Write DHEADER + content
        buffer.extend_from_slice(&(content.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&content);
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // Read DHEADER
        if data.len() < 4 {
            return None;
        }
        let _dheader = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        cursor += 4;

        // base_type: TypeIdentifier
        let (base_type, bytes_read) = TypeIdentifier::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        // detail: CompleteTypeDetail
        let (detail, bytes_read) = CompleteTypeDetail::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        Some((CompleteStructHeader { base_type, detail }, cursor))
    }
}

impl CompleteTypeDetail {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // ann_builtin: Option (1 byte present flag + data)
        if let Some(_ann) = &self.ann_builtin {
            buffer.push(1); // Present
                            // Serialize AppliedBuiltinTypeAnnotations (not implemented)
        } else {
            buffer.push(0); // Not present
        }

        // ann_custom: Option
        if let Some(_ann) = &self.ann_custom {
            buffer.push(1);
            // Serialize AppliedAnnotationSeq (not implemented)
            buffer.extend_from_slice(&0u32.to_le_bytes()); // Empty sequence
        } else {
            buffer.push(0);
        }

        // type_name: String (length + UTF-8 bytes + null terminator)
        serialize_string(&self.type_name, buffer);
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // ann_builtin: Option (1 byte present flag)
        if data.is_empty() {
            return None;
        }
        let ann_builtin_present = data[cursor] != 0;
        cursor += 1;
        let ann_builtin = if ann_builtin_present {
            // Skip annotation data (not implemented)
            None
        } else {
            None
        };

        // ann_custom: Option
        if cursor >= data.len() {
            return None;
        }
        let ann_custom_present = data[cursor] != 0;
        cursor += 1;
        let ann_custom = if ann_custom_present {
            // Skip sequence length
            if cursor + 4 > data.len() {
                return None;
            }
            cursor += 4;
            None
        } else {
            None
        };

        // type_name: String
        let (type_name, bytes_read) = deserialize_string(&data[cursor..])?;
        cursor += bytes_read;

        Some((
            CompleteTypeDetail {
                ann_builtin,
                ann_custom,
                type_name,
            },
            cursor,
        ))
    }
}

impl CompleteStructMember {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // APPENDABLE type needs DHEADER
        let mut content = Vec::new();

        // common: CommonStructMember
        self.common.serialize_to(&mut content);

        // detail: CompleteMemberDetail
        self.detail.serialize_to(&mut content);

        // Write DHEADER + content
        buffer.extend_from_slice(&(content.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&content);
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // Read DHEADER
        if data.len() < 4 {
            return None;
        }
        let _dheader = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        cursor += 4;

        // common: CommonStructMember
        let (common, bytes_read) = CommonStructMember::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        // detail: CompleteMemberDetail
        let (detail, bytes_read) = CompleteMemberDetail::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        Some((CompleteStructMember { common, detail }, cursor))
    }
}

impl CommonStructMember {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // member_id: u32
        buffer.extend_from_slice(&self.member_id.to_le_bytes());

        // member_flags: StructMemberFlag (u16)
        let flags = self.member_flags.to_u16();
        buffer.extend_from_slice(&flags.to_le_bytes());

        // member_type_id: TypeIdentifier
        self.member_type_id.serialize_to(buffer);
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // member_id: u32
        if data.len() < 4 {
            return None;
        }
        let member_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        cursor += 4;

        // member_flags: u16
        if data.len() < cursor + 2 {
            return None;
        }
        let flags_u16 = u16::from_le_bytes([data[cursor], data[cursor + 1]]);
        let member_flags = StructMemberFlag::from_u16(flags_u16);
        cursor += 2;

        // member_type_id: TypeIdentifier
        let (member_type_id, bytes_read) = TypeIdentifier::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        Some((
            CommonStructMember {
                member_id,
                member_flags,
                member_type_id,
            },
            cursor,
        ))
    }
}

impl StructMemberFlag {
    fn to_u16(&self) -> u16 {
        let mut flags = 0u16;
        match self.try_construct {
            TryConstructKind::Discard => flags |= 0x0001,
            TryConstructKind::UseDefault => flags |= 0x0002,
            TryConstructKind::Trim => flags |= 0x0003,
        }
        if self.is_external {
            flags |= 0x0004;
        }
        if self.is_optional {
            flags |= 0x0008;
        }
        if self.is_must_undestand {
            flags |= 0x0010;
        }
        if self.is_key {
            flags |= 0x0020;
        }
        flags
    }

    fn from_u16(flags: u16) -> Self {
        let try_construct = match flags & 0x0003 {
            0x0001 => TryConstructKind::Discard,
            0x0003 => TryConstructKind::Trim,
            _ => TryConstructKind::UseDefault,
        };
        StructMemberFlag {
            try_construct,
            is_external: (flags & 0x0004) != 0,
            is_optional: (flags & 0x0008) != 0,
            is_must_undestand: (flags & 0x0010) != 0,
            is_key: (flags & 0x0020) != 0,
        }
    }
}

impl CompleteMemberDetail {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // name: String
        serialize_string(&self.name, buffer);

        // ann_builtin: Option
        if let Some(_ann) = &self.ann_builtin {
            buffer.push(1);
            // Serialize AppliedBuiltinMemberAnnotations (not implemented)
        } else {
            buffer.push(0);
        }

        // ann_custom: Option
        if let Some(_ann) = &self.ann_custom {
            buffer.push(1);
            buffer.extend_from_slice(&0u32.to_le_bytes()); // Empty sequence
        } else {
            buffer.push(0);
        }
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // name: String
        let (name, bytes_read) = deserialize_string(&data[cursor..])?;
        cursor += bytes_read;

        // ann_builtin: Option
        if cursor >= data.len() {
            return None;
        }
        let ann_builtin_present = data[cursor] != 0;
        cursor += 1;
        let ann_builtin = if ann_builtin_present {
            // Skip annotation data (not implemented)
            None
        } else {
            None
        };

        // ann_custom: Option
        if cursor >= data.len() {
            return None;
        }
        let ann_custom_present = data[cursor] != 0;
        cursor += 1;
        let ann_custom = if ann_custom_present {
            // Skip sequence length
            if cursor + 4 > data.len() {
                return None;
            }
            cursor += 4;
            None
        } else {
            None
        };

        Some((
            CompleteMemberDetail {
                name,
                ann_builtin,
                ann_custom,
            },
            cursor,
        ))
    }
}

impl TypeIdentifier {
    /// Serializes this TypeIdentifier to bytes.
    pub fn serialize_to(&self, buffer: &mut Vec<u8>) {
        match self {
            // Primitive types - just the discriminator byte
            TypeIdentifier::TkNone => buffer.push(TypeKind::NONE as u8),
            TypeIdentifier::TkBoolean => buffer.push(TypeKind::BOOLEAN as u8),
            TypeIdentifier::TkByteType => buffer.push(TypeKind::BYTE as u8),
            TypeIdentifier::TkInt8Type => buffer.push(TypeKind::INT8 as u8),
            TypeIdentifier::TkInt16Type => buffer.push(TypeKind::INT16 as u8),
            TypeIdentifier::TkInt32Type => buffer.push(TypeKind::INT32 as u8),
            TypeIdentifier::TkInt64Type => buffer.push(TypeKind::INT64 as u8),
            TypeIdentifier::TkUint8Type => buffer.push(TypeKind::UINT8 as u8),
            TypeIdentifier::TkUint16Type => buffer.push(TypeKind::UINT16 as u8),
            TypeIdentifier::TkUint32Type => buffer.push(TypeKind::UINT32 as u8),
            TypeIdentifier::TkUint64Type => buffer.push(TypeKind::UINT64 as u8),
            TypeIdentifier::TkFloat32Type => buffer.push(TypeKind::FLOAT32 as u8),
            TypeIdentifier::TkFloat64Type => buffer.push(TypeKind::FLOAT64 as u8),
            TypeIdentifier::TkFloat128Type => buffer.push(TypeKind::FLOAT128 as u8),
            TypeIdentifier::TkChar8Type => buffer.push(TypeKind::CHAR8 as u8),
            TypeIdentifier::TkChar16Type => buffer.push(TypeKind::CHAR16 as u8),

            // String types
            TypeIdentifier::TiString8Small { string_sdefn } => {
                buffer.push(TI_STRING8_SMALL);
                buffer.push(string_sdefn.bound);
            }
            TypeIdentifier::TiString8Large { string_ldefn } => {
                buffer.push(TI_STRING8_LARGE);
                buffer.extend_from_slice(&string_ldefn.bound.to_le_bytes());
            }
            TypeIdentifier::TiString16Small { string_sdefn } => {
                buffer.push(TI_STRING16_SMALL);
                buffer.push(string_sdefn.bound);
            }
            TypeIdentifier::TiString16Large { string_ldefn } => {
                buffer.push(TI_STRING16_LARGE);
                buffer.extend_from_slice(&string_ldefn.bound.to_le_bytes());
            }

            // Sequence types
            TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
                buffer.push(TI_PLAIN_SEQUENCE_SMALL);
                seq_sdefn.header.serialize_to(buffer);
                buffer.push(seq_sdefn.bound);
                seq_sdefn.element_identifier.serialize_to(buffer);
            }
            TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
                buffer.push(TI_PLAIN_SEQUENCE_LARGE);
                seq_ldefn.header.serialize_to(buffer);
                buffer.extend_from_slice(&seq_ldefn.bound.to_le_bytes());
                seq_ldefn.element_identifier.serialize_to(buffer);
            }

            // Array types
            TypeIdentifier::TiPlainArraySmall { array_sdefn } => {
                buffer.push(TI_PLAIN_ARRAY_SMALL);
                array_sdefn.header.serialize_to(buffer);
                buffer.extend_from_slice(&(array_sdefn.array_bound_seq.len() as u32).to_le_bytes());
                for &bound in &array_sdefn.array_bound_seq {
                    buffer.push(bound);
                }
                array_sdefn.element_identifier.serialize_to(buffer);
            }
            TypeIdentifier::TiPlainArrayLarge { array_ldefn } => {
                buffer.push(TI_PLAIN_ARRAY_LARGE);
                array_ldefn.header.serialize_to(buffer);
                buffer.extend_from_slice(&(array_ldefn.array_bound_seq.len() as u32).to_le_bytes());
                for &bound in &array_ldefn.array_bound_seq {
                    buffer.extend_from_slice(&bound.to_le_bytes());
                }
                array_ldefn.element_identifier.serialize_to(buffer);
            }

            // Map types
            TypeIdentifier::TiPlainMapSmall { map_sdefn } => {
                buffer.push(TI_PLAIN_MAP_SMALL);
                map_sdefn.header.serialize_to(buffer);
                buffer.push(map_sdefn.bound);
                map_sdefn.element_identifier.serialize_to(buffer);
                map_sdefn.key_flags.serialize_to(buffer);
                map_sdefn.key_identifier.serialize_to(buffer);
            }
            TypeIdentifier::TiPlainMapLarge { map_ldefn } => {
                buffer.push(TI_PLAIN_MAP_LARGE);
                map_ldefn.header.serialize_to(buffer);
                buffer.extend_from_slice(&map_ldefn.bound.to_le_bytes());
                map_ldefn.element_identifier.serialize_to(buffer);
                map_ldefn.key_flags.serialize_to(buffer);
                map_ldefn.key_identifier.serialize_to(buffer);
            }

            // Strongly connected component
            TypeIdentifier::TiStronglyConnectedComponent { sc_component_id } => {
                buffer.push(TI_STRONGLY_CONNECTED_COMPONENT);
                // TypeObjectHashId
                match &sc_component_id.sc_component_id {
                    TypeObjectHashId::EkComplete { hash } => {
                        buffer.push(EK_COMPLETE);
                        buffer.extend_from_slice(hash);
                    }
                    TypeObjectHashId::EkMinimal { hash } => {
                        buffer.push(EK_MINIMAL);
                        buffer.extend_from_slice(hash);
                    }
                }
                buffer.extend_from_slice(&sc_component_id.scc_length.to_le_bytes());
                buffer.extend_from_slice(&sc_component_id.scc_index.to_le_bytes());
            }

            // Hash-based type references
            TypeIdentifier::EkComplete { complete } => {
                buffer.push(EK_COMPLETE);
                // For EkComplete, we should serialize the hash of the type
                // For now, serialize a placeholder - full implementation would compute hash
                let type_object = complete.to_type_object().ok();
                if let Some(to) = type_object {
                    // Compute MD5 hash of serialized TypeObject
                    let serialized = to.serialize_to_bytes();
                    let hash = md5::compute(&serialized);
                    buffer.extend_from_slice(&hash[0..14]);
                } else {
                    buffer.extend_from_slice(&[0u8; 14]);
                }
            }
            TypeIdentifier::EkMinimal { .. } => {
                buffer.push(EK_MINIMAL);
                buffer.extend_from_slice(&[0u8; 14]); // Placeholder hash
            }

            // Hash-only variants (from wire deserialization)
            TypeIdentifier::EkCompleteHash { hash } => {
                buffer.push(EK_COMPLETE);
                buffer.extend_from_slice(hash);
            }
            TypeIdentifier::EkMinimalHash { hash } => {
                buffer.push(EK_MINIMAL);
                buffer.extend_from_slice(hash);
            }
        }
    }

    /// Deserializes a TypeIdentifier from bytes.
    pub fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        if data.is_empty() {
            return None;
        }

        let discriminator = data[0];
        let mut cursor = 1;

        match discriminator {
            // Primitive types
            x if x == TypeKind::NONE as u8 => Some((TypeIdentifier::TkNone, cursor)),
            x if x == TypeKind::BOOLEAN as u8 => Some((TypeIdentifier::TkBoolean, cursor)),
            x if x == TypeKind::BYTE as u8 => Some((TypeIdentifier::TkByteType, cursor)),
            x if x == TypeKind::INT8 as u8 => Some((TypeIdentifier::TkInt8Type, cursor)),
            x if x == TypeKind::INT16 as u8 => Some((TypeIdentifier::TkInt16Type, cursor)),
            x if x == TypeKind::INT32 as u8 => Some((TypeIdentifier::TkInt32Type, cursor)),
            x if x == TypeKind::INT64 as u8 => Some((TypeIdentifier::TkInt64Type, cursor)),
            x if x == TypeKind::UINT8 as u8 => Some((TypeIdentifier::TkUint8Type, cursor)),
            x if x == TypeKind::UINT16 as u8 => Some((TypeIdentifier::TkUint16Type, cursor)),
            x if x == TypeKind::UINT32 as u8 => Some((TypeIdentifier::TkUint32Type, cursor)),
            x if x == TypeKind::UINT64 as u8 => Some((TypeIdentifier::TkUint64Type, cursor)),
            x if x == TypeKind::FLOAT32 as u8 => Some((TypeIdentifier::TkFloat32Type, cursor)),
            x if x == TypeKind::FLOAT64 as u8 => Some((TypeIdentifier::TkFloat64Type, cursor)),
            x if x == TypeKind::FLOAT128 as u8 => Some((TypeIdentifier::TkFloat128Type, cursor)),
            x if x == TypeKind::CHAR8 as u8 => Some((TypeIdentifier::TkChar8Type, cursor)),
            x if x == TypeKind::CHAR16 as u8 => Some((TypeIdentifier::TkChar16Type, cursor)),

            // String types
            TI_STRING8_SMALL => {
                if data.len() < cursor + 1 {
                    return None;
                }
                let bound = data[cursor];
                cursor += 1;
                Some((
                    TypeIdentifier::TiString8Small {
                        string_sdefn: StringSTypeDefn { bound },
                    },
                    cursor,
                ))
            }
            TI_STRING8_LARGE => {
                if data.len() < cursor + 4 {
                    return None;
                }
                let bound = u32::from_le_bytes([
                    data[cursor],
                    data[cursor + 1],
                    data[cursor + 2],
                    data[cursor + 3],
                ]);
                cursor += 4;
                Some((
                    TypeIdentifier::TiString8Large {
                        string_ldefn: StringLTypeDefn { bound },
                    },
                    cursor,
                ))
            }

            // Sequence types
            TI_PLAIN_SEQUENCE_LARGE => {
                // header: PlainCollectionHeader
                let (header, bytes_read) = PlainCollectionHeader::deserialize_from(&data[cursor..])?;
                cursor += bytes_read;

                // bound: u32
                if data.len() < cursor + 4 {
                    return None;
                }
                let bound = u32::from_le_bytes([
                    data[cursor],
                    data[cursor + 1],
                    data[cursor + 2],
                    data[cursor + 3],
                ]);
                cursor += 4;

                // element_identifier: TypeIdentifier
                let (element_identifier, bytes_read) =
                    TypeIdentifier::deserialize_from(&data[cursor..])?;
                cursor += bytes_read;

                Some((
                    TypeIdentifier::TiPlainSequenceLarge {
                        seq_ldefn: Box::new(PlainSequenceLElemDefn {
                            header,
                            bound,
                            element_identifier,
                        }),
                    },
                    cursor,
                ))
            }

            // Hash-based type references - just the hash
            EK_COMPLETE => {
                // For wire format, these are just the 14-byte hash
                if data.len() < cursor + 14 {
                    return None;
                }
                let mut hash = [0u8; 14];
                hash.copy_from_slice(&data[cursor..cursor + 14]);
                cursor += 14;
                Some((TypeIdentifier::EkCompleteHash { hash }, cursor))
            }
            EK_MINIMAL => {
                // For wire format, these are just the 14-byte hash
                if data.len() < cursor + 14 {
                    return None;
                }
                let mut hash = [0u8; 14];
                hash.copy_from_slice(&data[cursor..cursor + 14]);
                cursor += 14;
                Some((TypeIdentifier::EkMinimalHash { hash }, cursor))
            }

            // Other types not yet supported
            _ => None,
        }
    }
}

impl PlainCollectionHeader {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        buffer.push(self.equiv_kind);
        self.element_flags.serialize_to(buffer);
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // equiv_kind: u8
        if data.is_empty() {
            return None;
        }
        let equiv_kind = data[cursor];
        cursor += 1;

        // element_flags: CollectionElementFlag (u16)
        let (element_flags, bytes_read) = CollectionElementFlag::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        Some((
            PlainCollectionHeader {
                equiv_kind,
                element_flags,
            },
            cursor,
        ))
    }
}

impl CollectionElementFlag {
    fn serialize_to(&self, buffer: &mut Vec<u8>) {
        let mut flags = 0u16;
        match self.try_construct {
            TryConstructKind::Discard => flags |= 0x0001,
            TryConstructKind::UseDefault => flags |= 0x0002,
            TryConstructKind::Trim => flags |= 0x0003,
        }
        if self.is_external {
            flags |= 0x0004;
        }
        buffer.extend_from_slice(&flags.to_le_bytes());
    }

    fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        if data.len() < 2 {
            return None;
        }
        let flags = u16::from_le_bytes([data[0], data[1]]);
        let try_construct = match flags & 0x0003 {
            0x0001 => TryConstructKind::Discard,
            0x0003 => TryConstructKind::Trim,
            _ => TryConstructKind::UseDefault,
        };
        Some((
            CollectionElementFlag {
                try_construct,
                is_external: (flags & 0x0004) != 0,
            },
            2,
        ))
    }
}

/// Helper function to serialize a string in CDR format.
fn serialize_string(s: &str, buffer: &mut Vec<u8>) {
    let bytes = s.as_bytes();
    // String length including null terminator
    buffer.extend_from_slice(&((bytes.len() + 1) as u32).to_le_bytes());
    buffer.extend_from_slice(bytes);
    buffer.push(0); // Null terminator
}

/// Helper function to deserialize a string from CDR format.
fn deserialize_string(data: &[u8]) -> Option<(String, usize)> {
    if data.len() < 4 {
        return None;
    }
    let length = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if length == 0 || data.len() < 4 + length {
        return None;
    }
    // String includes null terminator, so actual string is length-1 bytes
    let string_bytes = &data[4..4 + length - 1];
    let string = String::from_utf8(string_bytes.to_vec()).ok()?;
    Some((string, 4 + length))
}

// ============================================================================
// TypeInformation Serialization for Discovery
// ============================================================================

impl TypeIdentifierWithSize {
    /// Serializes this TypeIdentifierWithSize to bytes (FINAL struct).
    pub fn serialize_to(&self, buffer: &mut Vec<u8>) {
        // type_id: TypeIdentifier
        self.type_id.serialize_to(buffer);
        // typeobject_serialized_size: u32
        buffer.extend_from_slice(&self.typeobject_serialized_size.to_le_bytes());
    }

    /// Deserializes a TypeIdentifierWithSize from bytes.
    pub fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // type_id: TypeIdentifier
        let (type_id, bytes_read) = TypeIdentifier::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        // typeobject_serialized_size: u32
        if data.len() < cursor + 4 {
            return None;
        }
        let typeobject_serialized_size = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]);
        cursor += 4;

        Some((
            TypeIdentifierWithSize {
                type_id,
                typeobject_serialized_size,
            },
            cursor,
        ))
    }
}

impl TypeIdentifierWithDependencies {
    /// Serializes this TypeIdentifierWithDependencies to bytes (APPENDABLE struct with DHEADER).
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut inner = Vec::new();

        // typeid_with_size
        self.typeid_with_size.serialize_to(&mut inner);

        // dependent_typeid_count: i32
        inner.extend_from_slice(&self.dependent_typeid_count.to_le_bytes());

        // dependent_typeids: sequence
        inner.extend_from_slice(&(self.dependent_typeids.len() as u32).to_le_bytes());
        for dep in &self.dependent_typeids {
            dep.serialize_to(&mut inner);
        }

        // Prepend DHEADER
        let mut result = Vec::with_capacity(4 + inner.len());
        result.extend_from_slice(&(inner.len() as u32).to_le_bytes());
        result.extend_from_slice(&inner);
        result
    }

    /// Deserializes a TypeIdentifierWithDependencies from bytes.
    pub fn deserialize_from(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // Read DHEADER
        if data.len() < 4 {
            return None;
        }
        let dheader = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        cursor += 4;

        if data.len() < cursor + dheader {
            return None;
        }

        // typeid_with_size
        let (typeid_with_size, bytes_read) = TypeIdentifierWithSize::deserialize_from(&data[cursor..])?;
        cursor += bytes_read;

        // dependent_typeid_count: i32
        if data.len() < cursor + 4 {
            return None;
        }
        let dependent_typeid_count = i32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]);
        cursor += 4;

        // dependent_typeids: sequence
        if data.len() < cursor + 4 {
            return None;
        }
        let dep_count = u32::from_le_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        cursor += 4;

        let mut dependent_typeids = Vec::with_capacity(dep_count);
        for _ in 0..dep_count {
            let (dep, bytes_read) = TypeIdentifierWithSize::deserialize_from(&data[cursor..])?;
            cursor += bytes_read;
            dependent_typeids.push(dep);
        }

        Some((
            TypeIdentifierWithDependencies {
                typeid_with_size,
                dependent_typeid_count,
                dependent_typeids,
            },
            cursor,
        ))
    }
}

// Parameter IDs for TypeInformation (MUTABLE struct)
const PID_TYPE_INFO_MINIMAL: u32 = 0x1001;
const PID_TYPE_INFO_COMPLETE: u32 = 0x1002;
const PID_SENTINEL: u32 = 0x0001;

impl TypeInformation {
    /// Serializes this TypeInformation to bytes for discovery.
    ///
    /// TypeInformation is MUTABLE, so uses parameter list encoding:
    /// - DHEADER: 4 bytes total size
    /// - For each field: EMHEADER (4 bytes PID + must_understand flag) + NEXTINT (4 bytes length) + data
    /// - Sentinel: PID_SENTINEL (special marker)
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let mut inner = Vec::new();

        // minimal field (PID 0x1001)
        let minimal_bytes = self.minimal.serialize_to_bytes();
        // EMHEADER: PID with must_understand=false (bit 31 clear)
        inner.extend_from_slice(&PID_TYPE_INFO_MINIMAL.to_le_bytes());
        // NEXTINT: field size
        inner.extend_from_slice(&(minimal_bytes.len() as u32).to_le_bytes());
        inner.extend_from_slice(&minimal_bytes);

        // complete field (PID 0x1002)
        let complete_bytes = self.complete.serialize_to_bytes();
        inner.extend_from_slice(&PID_TYPE_INFO_COMPLETE.to_le_bytes());
        inner.extend_from_slice(&(complete_bytes.len() as u32).to_le_bytes());
        inner.extend_from_slice(&complete_bytes);

        // Sentinel
        inner.extend_from_slice(&PID_SENTINEL.to_le_bytes());
        inner.extend_from_slice(&0u32.to_le_bytes()); // Sentinel has length 0

        // Prepend DHEADER
        let mut result = Vec::with_capacity(4 + inner.len());
        result.extend_from_slice(&(inner.len() as u32).to_le_bytes());
        result.extend_from_slice(&inner);
        result
    }

    /// Deserializes TypeInformation from bytes.
    pub fn deserialize_from_bytes(data: &[u8]) -> Option<(Self, usize)> {
        let mut cursor = 0;

        // Read DHEADER
        if data.len() < 4 {
            return None;
        }
        let dheader = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        cursor += 4;

        if data.len() < cursor + dheader {
            return None;
        }

        let mut minimal = None;
        let mut complete = None;

        // Parse parameter list
        while cursor < data.len() {
            if data.len() < cursor + 4 {
                break;
            }
            let pid = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]);
            cursor += 4;

            if pid == PID_SENTINEL {
                cursor += 4; // Skip sentinel length
                break;
            }

            if data.len() < cursor + 4 {
                break;
            }
            let length = u32::from_le_bytes([data[cursor], data[cursor + 1], data[cursor + 2], data[cursor + 3]]) as usize;
            cursor += 4;

            match pid & 0x0FFFFFFF {
                // Mask out flags
                x if x == PID_TYPE_INFO_MINIMAL => {
                    if let Some((m, _)) = TypeIdentifierWithDependencies::deserialize_from(&data[cursor..]) {
                        minimal = Some(m);
                    }
                }
                x if x == PID_TYPE_INFO_COMPLETE => {
                    if let Some((c, _)) = TypeIdentifierWithDependencies::deserialize_from(&data[cursor..]) {
                        complete = Some(c);
                    }
                }
                _ => {} // Skip unknown PIDs
            }
            cursor += length;
        }

        Some((
            TypeInformation {
                minimal: minimal.unwrap_or_else(|| TypeIdentifierWithDependencies {
                    typeid_with_size: TypeIdentifierWithSize {
                        type_id: TypeIdentifier::TkNone,
                        typeobject_serialized_size: 0,
                    },
                    dependent_typeid_count: 0,
                    dependent_typeids: Vec::new(),
                }),
                complete: complete.unwrap_or_else(|| TypeIdentifierWithDependencies {
                    typeid_with_size: TypeIdentifierWithSize {
                        type_id: TypeIdentifier::TkNone,
                        typeobject_serialized_size: 0,
                    },
                    dependent_typeid_count: 0,
                    dependent_typeids: Vec::new(),
                }),
            },
            cursor,
        ))
    }

    /// Extracts the complete TypeIdentifier bytes from this TypeInformation.
    ///
    /// This is used for TypeLookup requests - we need to send the TypeIdentifier
    /// to the remote participant to request the full TypeObject.
    pub fn get_complete_type_identifier_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        self.complete.typeid_with_size.type_id.serialize_to(&mut buffer);
        buffer
    }
}
