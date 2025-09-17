use crate::xtypes::type_object::TypeObject;

use super::{
    error::XTypesError,
    type_object::{TypeIdentifier, TypeKind},
};
use alloc::string::String;

pub type ObjectName = String;

#[derive(Clone, Copy)]
pub enum ExtensibilityKind {
    Final,
    Appendable,
    Mutable,
}
#[derive(Clone, Copy)]
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

pub struct MemberDescriptor {
    pub name: ObjectName,
    pub id: MemberId,
    pub r#type: TypeIdentifier,
    pub default_value: &'static str,
    pub index: u32,
    // pub label :UnionCaseLabelSeq,
    pub try_construct_kind: TryConstructKind,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
    pub is_shared: bool,
    pub is_default_label: bool,
}

pub struct DynamicType {
    descriptor: TypeDescriptor,
    name: ObjectName,
    kind: TypeKind,
}

impl DynamicType {
    pub fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        todo!()
    }
    pub fn get_name(&self) -> ObjectName {
        todo!()
    }
    pub fn get_kind(&self) -> TypeKind {
        todo!()
    }

    // DDS::ReturnCode_t get_member_by_name(inout DynamicTypeMember member, in ObjectName name);
    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    // DDS::ReturnCode_t get_member(inout DynamicTypeMember member, in MemberId id);
    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);
    pub fn get_member_count(&self) -> u32 {
        todo!()
    }
    pub fn get_member_by_index(&self, index: u32) -> Result<&DynamicTypeMember, XTypesError> {
        todo!()
    }
    // fn get_annotation_count(&self) -> u32;
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);
}

pub struct DynamicTypeMember {
    id: MemberId,
    name: ObjectName,
    descriptor: MemberDescriptor,
}

impl DynamicTypeMember {
    pub fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        todo!()
    }
    // unsigned long get_annotation_count();
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);

    pub fn get_id(&self) -> MemberId {
        self.id
    }
    pub fn get_name(&self) -> &ObjectName {
        &self.name
    }
}

pub struct DynamicTypeBuilderFactory;

impl DynamicTypeBuilderFactory {
    pub fn get_primitive_type(kind: TypeKind) -> DynamicType {
        todo!()
    }

    pub fn create_type(descriptor: TypeDescriptor) -> DynamicTypeBuilder {
        DynamicTypeBuilder
    }

    pub fn create_type_copy(r#type: DynamicType) -> DynamicTypeBuilder {
        DynamicTypeBuilder
    }

    pub fn create_type_w_type_object(type_object: TypeObject) -> DynamicTypeBuilder {
        DynamicTypeBuilder
    }

    pub fn create_string_type(bound: u32) -> DynamicTypeBuilder {
        DynamicTypeBuilder
    }

    pub fn create_sequence_type(bound: u32) -> DynamicTypeBuilder {
        DynamicTypeBuilder
    }
}

pub struct DynamicTypeBuilder;

impl DynamicTypeBuilder {}

pub struct DynamicData;
