use core::any::Any;

use crate::xtypes::{
    error::XTypesResult,
    type_object::{LBoundSeq, TypeObject},
};

use super::{
    error::XTypesError,
    type_object::{TypeIdentifier, TypeKind},
};
use alloc::{collections::BTreeMap, string::String, vec::Vec};

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
    member_list: Vec<DynamicTypeMember>,
}

impl DynamicType {
    pub fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        todo!()
    }
    pub fn get_name(&self) -> ObjectName {
        self.descriptor.name.clone()
    }
    pub fn get_kind(&self) -> TypeKind {
        self.descriptor.kind
    }

    // DDS::ReturnCode_t get_member_by_name(inout DynamicTypeMember member, in ObjectName name);
    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    // DDS::ReturnCode_t get_member(inout DynamicTypeMember member, in MemberId id);
    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);

    pub fn get_member_count(&self) -> u32 {
        self.member_list.len() as u32
    }
    pub fn get_member_by_index(&self, index: u32) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .get(index as usize)
            .ok_or(XTypesError::InvalidIndex)
    }
    // fn get_annotation_count(&self) -> u32;
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);
}

pub struct DynamicTypeMember {
    descriptor: MemberDescriptor,
}

impl DynamicTypeMember {
    pub fn get_descriptor(&self) -> XTypesResult<&MemberDescriptor> {
        Ok(&self.descriptor)
    }
    // unsigned long get_annotation_count();
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);

    pub fn get_id(&self) -> MemberId {
        self.descriptor.id
    }
    pub fn get_name(&self) -> &ObjectName {
        &self.descriptor.name
    }
}

pub struct DynamicTypeBuilderFactory;

impl DynamicTypeBuilderFactory {
    pub fn get_primitive_type(kind: TypeKind) -> DynamicType {
        let kind = match kind {
            TypeKind::BOOLEAN
            | TypeKind::BYTE
            | TypeKind::INT16
            | TypeKind::INT32
            | TypeKind::INT64
            | TypeKind::UINT16
            | TypeKind::UINT32
            | TypeKind::UINT64
            | TypeKind::FLOAT32
            | TypeKind::FLOAT64
            | TypeKind::INT8
            | TypeKind::UINT8
            | TypeKind::CHAR8
            | TypeKind::STRING8 => kind,
            TypeKind::NONE
            | TypeKind::CHAR16  // Not available in Rust type system
            | TypeKind::STRING16 // Not available in Rust type system
            | TypeKind::FLOAT128 // Not available in Rust type system
            | TypeKind::ALIAS
            | TypeKind::ENUM
            | TypeKind::BITMASK
            | TypeKind::ANNOTATION
            | TypeKind::STRUCTURE
            | TypeKind::UNION
            | TypeKind::BITSET
            | TypeKind::SEQUENCE
            | TypeKind::ARRAY
            | TypeKind::MAP => TypeKind::NONE,
        };
        DynamicType {
            descriptor: TypeDescriptor {
                kind,
                name: String::from(""),
                extensibility_kind: ExtensibilityKind::Appendable,
                is_nested: false,
            },
            member_list: vec![],
        }
    }

    pub fn create_type(descriptor: TypeDescriptor) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor,
            member_list: Vec::new(),
        }
    }

    pub fn create_type_copy(r#type: DynamicType) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_type_object(type_object: TypeObject) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_string_type(bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_wstring_type(bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_sequence_type(element_type: DynamicType, bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_array_type(element_type: DynamicType, bound: LBoundSeq) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_map_type(
        key_element_type: DynamicType,
        element_type: DynamicType,
        bound: u32,
    ) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_bitmask_type(bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_uri(
        document_url: String,
        type_name: String,
        include_paths: Vec<String>,
    ) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_document(
        document: String,
        type_name: String,
        include_paths: Vec<String>,
    ) -> DynamicTypeBuilder {
        todo!()
    }
}

pub struct DynamicTypeBuilder {
    descriptor: TypeDescriptor,
    member_list: Vec<MemberDescriptor>,
}

impl DynamicTypeBuilder {
    pub fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        todo!()
    }

    pub fn get_name(&self) -> ObjectName {
        todo!()
    }

    pub fn get_kind(&self) -> TypeKind {
        todo!()
    }

    pub fn get_member_by_name(&self, name: &ObjectName) -> Result<DynamicTypeMember, XTypesError> {
        todo!()
    }

    pub fn get_all_members_by_name(
        &self,
    ) -> Result<Vec<(ObjectName, DynamicTypeMember)>, XTypesError> {
        todo!()
    }

    pub fn get_member(&self, id: MemberId) -> Result<DynamicTypeMember, XTypesError> {
        todo!()
    }

    pub fn get_all_members(&self) -> Result<Vec<(MemberId, DynamicTypeMember)>, XTypesError> {
        todo!()
    }

    pub fn get_member_count(&self) -> u32 {
        todo!()
    }

    pub fn get_member_by_index(&self, index: u32) -> XTypesResult<DynamicTypeMember> {
        todo!()
    }

    pub fn get_annotation_count(&self) -> u32 {
        todo!()
    }

    pub fn get_annotation(&self, idx: u32) -> XTypesResult<()> {
        todo!()
    }

    pub fn add_member(&mut self, descriptor: MemberDescriptor) -> XTypesResult<()> {
        if !matches!(
            self.descriptor.kind,
            TypeKind::ENUM
                | TypeKind::BITMASK
                | TypeKind::ANNOTATION
                | TypeKind::STRUCTURE
                | TypeKind::UNION
                | TypeKind::BITSET
        ) {
            return Err(XTypesError::IllegalOperation);
        }

        self.member_list.push(descriptor);

        Ok(())
    }

    pub fn apply_annotation(&mut self) -> XTypesResult<()> {
        todo!()
    }

    pub fn apply_annotation_to_member(&mut self, id: MemberId) -> XTypesResult<()> {
        todo!()
    }

    pub fn set_name(&mut self, name: ObjectName) -> XTypesResult<()> {
        todo!()
    }

    pub fn build(self) -> DynamicType {
        DynamicType {
            descriptor: self.descriptor,
            member_list: self
                .member_list
                .into_iter()
                .map(|descriptor| DynamicTypeMember { descriptor })
                .collect(),
        }
    }
}

pub struct DynamicData<'a> {
    type_ref: &'a DynamicType,
    abstract_data: BTreeMap<MemberId, Box<dyn Any>>,
}

impl<'a> DynamicData<'a> {
    pub fn get_int64_value(&self, id: MemberId) -> XTypesResult<&i64> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref::<i64>()
            .ok_or(XTypesError::InvalidData)
    }
}
