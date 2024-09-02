use crate::error::XcdrError;

pub type ObjectName = &'static str;
pub type TypeKind = u8;

// Primitive TKs
pub const TK_NONE: TypeKind = 0x00;
pub const TK_BOOLEAN: TypeKind = 0x01;
pub const TK_BYTE: TypeKind = 0x02;
pub const TK_INT16: TypeKind = 0x03;
pub const TK_INT32: TypeKind = 0x04;
pub const TK_INT64: TypeKind = 0x05;
pub const TK_UINT16: TypeKind = 0x06;
pub const TK_UINT32: TypeKind = 0x07;
pub const TK_UINT64: TypeKind = 0x08;
pub const TK_FLOAT32: TypeKind = 0x09;
pub const TK_FLOAT64: TypeKind = 0x0A;
pub const TK_FLOAT128: TypeKind = 0x0B;
pub const TK_INT8: TypeKind = 0x0C;
pub const TK_UINT8: TypeKind = 0x0D;
pub const TK_CHAR8: TypeKind = 0x10;
pub const TK_CHAR16: TypeKind = 0x11;
// String TKs
pub const TK_STRING8: TypeKind = 0x20;
pub const TK_STRING16: TypeKind = 0x21;
// Constructed/Named types
pub const TK_ALIAS: TypeKind = 0x30;
// Enumerated TKs
pub const TK_ENUM: TypeKind = 0x40;
pub const TK_BITMASK: TypeKind = 0x41;
// Structured TKs
pub const TK_ANNOTATION: TypeKind = 0x50;
pub const TK_STRUCTURE: TypeKind = 0x51;
pub const TK_UNION: TypeKind = 0x52;
pub const TK_BITSET: TypeKind = 0x53;
// Collection TKs
pub const TK_SEQUENCE: TypeKind = 0x60;
pub const TK_ARRAY: TypeKind = 0x61;
pub const TK_MAP: TypeKind = 0x62;
// ---------- TypeKinds (end) -------------------

pub enum ExtensibilityKind {
    Final,
    Appendable,
    Mutable,
}
pub enum TryConstructKind {
    UseDefault,
    Discard,
    Trim,
}

pub struct TypeDescriptor {
    pub kind: TypeKind,
    pub name: ObjectName,
    // pub DynamicType base_type;
    // pub DynamicType discriminator_type;
    // pub bound: BoundSeq
    // @optional public DynamicType element_type;
    // @optional public DynamicType key_element_type;
    pub extensibility_kind: ExtensibilityKind,
    pub is_nested: bool,
}

pub type MemberId = u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MemberDescriptor {
    pub name: ObjectName,
    pub id: MemberId,
    // pub _type: &dyn DynamicType,
    pub default_value: &'static str,
    pub index: u32,
    // pub label :UnionCaseLabelSeq,
    // pub try_construct_kind: TryConstructKind,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
    pub is_shared: bool,
    pub is_default_label: bool,
}

pub trait DynamicType {
    fn get_descriptor(&self) -> Result<TypeDescriptor, XcdrError>;
    fn get_name(&self) -> ObjectName;
    fn get_kind(&self) -> TypeKind;

    // DDS::ReturnCode_t get_member_by_name(inout DynamicTypeMember member, in ObjectName name);
    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    // DDS::ReturnCode_t get_member(inout DynamicTypeMember member, in MemberId id);
    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);
    fn get_member_count(&self) -> u32;
    fn get_member_by_index(&self, index: u32) -> Result<impl DynamicTypeMember, XcdrError>;
    // fn get_annotation_count(&self) -> u32;
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);
}

pub trait DynamicTypeMember {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XcdrError>;
    // unsigned long get_annotation_count();
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);

    fn get_id(&self) -> MemberId;
    fn get_name(&self) -> ObjectName;
}

impl DynamicTypeMember for MemberDescriptor {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XcdrError> {
        Ok(self.clone())
    }

    fn get_id(&self) -> MemberId {
        self.id
    }

    fn get_name(&self) -> ObjectName {
        self.name
    }
}
