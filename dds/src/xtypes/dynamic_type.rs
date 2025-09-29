use core::any::Any;

use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{error::XTypesResult, type_object::TypeObject},
};

use super::error::XTypesError;
use alloc::{collections::BTreeMap, string::String, vec::Vec};

pub type BoundSeq = Vec<u32>;
pub type IncludePathSeq = Vec<String>;
pub type ObjectName = String;

// ---------- TypeKinds (begin) -------------------
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
// String TK;
pub const TK_STRING8: TypeKind = 0x20;
pub const TK_STRING16: TypeKind = 0x21;
// Constructed/Named type;
pub const TK_ALIAS: TypeKind = 0x30;
// Enumerated TK;
pub const TK_ENUM: TypeKind = 0x40;
pub const TK_BITMASK: TypeKind = 0x41;
// Structured TK;
pub const TK_ANNOTATION: TypeKind = 0x50;
pub const TK_STRUCTURE: TypeKind = 0x51;
pub const TK_UNION: TypeKind = 0x52;
pub const TK_BITSET: TypeKind = 0x53;
// Collection TK;
pub const TK_SEQUENCE: TypeKind = 0x60;
pub const TK_ARRAY: TypeKind = 0x61;
pub const TK_MAP: TypeKind = 0x62;

// ---------- TypeKinds (end) -------------------

pub struct DynamicTypeBuilderFactory;

impl DynamicTypeBuilderFactory {
    pub fn get_primitive_type(kind: TypeKind) -> DynamicType {
        DynamicType {
            descriptor: Box::new(TypeDescriptor {
                kind,
                name: String::new(),
                base_type: None,
                discriminator_type: None,
                bound: Vec::new(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            }),
            member_list: Vec::new(),
        }
    }

    pub fn create_type(descriptor: TypeDescriptor) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor,
            member_list: Vec::new(),
        }
    }

    pub fn create_type_copy(r#_type: DynamicType) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_type_object(_type_object: TypeObject) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_string_type(bound: u32) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TK_STRING8,
                name: String::new(),
                base_type: None,
                discriminator_type: None,
                bound: vec![bound],
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            member_list: Vec::new(),
        }
    }

    pub fn create_wstring_type(_bound: u32) -> DynamicTypeBuilder {
        unimplemented!("wstring not supported in Rust")
    }

    pub fn create_sequence_type(element_type: DynamicType, bound: u32) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TK_SEQUENCE,
                name: String::new(),
                base_type: None,
                discriminator_type: None,
                bound: vec![bound],
                element_type: Some(element_type),
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            member_list: Vec::new(),
        }
    }

    pub fn create_array_type(element_type: DynamicType, bound: BoundSeq) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TK_ARRAY,
                name: String::new(),
                base_type: None,
                discriminator_type: None,
                bound,
                element_type: Some(element_type),
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            member_list: Vec::new(),
        }
    }

    pub fn create_map_type(
        _key_element_type: DynamicType,
        _element_type: DynamicType,
        _bound: u32,
    ) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_bitmask_type(_bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_uri(
        _document_url: String,
        _type_name: String,
        _include_paths: Vec<String>,
    ) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_document(
        _document: String,
        _type_name: String,
        _include_paths: Vec<String>,
    ) -> DynamicTypeBuilder {
        todo!()
    }
}

pub type Parameters = BTreeMap<ObjectName, ObjectName>;

#[derive(Debug, Clone, Copy)]
pub enum ExtensibilityKind {
    Final,
    Appendable,
    Mutable,
}

#[derive(Debug, Clone, Copy)]
pub enum TryConstructKind {
    UseDefault,
    Discard,
    Trim,
}

#[derive(Debug, Clone)]
pub struct TypeDescriptor {
    pub kind: TypeKind,
    pub name: ObjectName,
    pub base_type: Option<DynamicType>,
    pub discriminator_type: Option<DynamicType>,
    pub bound: BoundSeq,
    pub element_type: Option<DynamicType>,
    pub key_element_type: Option<DynamicType>,
    pub extensibility_kind: ExtensibilityKind,
    pub is_nested: bool,
}

pub type MemberId = u32;
pub type UnionCaseLabelSeq = Vec<i32>;

#[derive(Debug, Clone)]
pub struct MemberDescriptor {
    pub name: ObjectName,
    pub id: MemberId,
    pub r#type: DynamicType,
    pub default_value: String,
    pub index: u32,
    pub label: UnionCaseLabelSeq,
    pub try_construct_kind: TryConstructKind,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
    pub is_shared: bool,
    pub is_default_label: bool,
}

#[derive(Debug, Clone)]
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

pub struct DynamicTypeBuilder {
    descriptor: TypeDescriptor,
    member_list: Vec<DynamicTypeMember>,
}

impl DynamicTypeBuilder {
    pub fn get_descriptor(&self) -> XTypesResult<&TypeDescriptor> {
        Ok(&self.descriptor)
    }

    pub fn get_name(&self) -> ObjectName {
        self.descriptor.name.clone()
    }

    pub fn get_kind(&self) -> TypeKind {
        self.descriptor.kind
    }

    pub fn get_member_by_name(
        &mut self,
        name: &ObjectName,
    ) -> XTypesResult<&mut DynamicTypeMember> {
        self.member_list
            .iter_mut()
            .find(|m| &m.descriptor.name == name)
            .ok_or(XTypesError::InvalidData)
    }

    pub fn get_all_members_by_name(
        &self,
    ) -> Result<Vec<(ObjectName, DynamicTypeMember)>, XTypesError> {
        todo!()
    }

    pub fn get_member(&self, _id: MemberId) -> Result<DynamicTypeMember, XTypesError> {
        todo!()
    }

    pub fn get_all_members(&self) -> Result<Vec<(MemberId, DynamicTypeMember)>, XTypesError> {
        todo!()
    }

    pub fn get_annotation_count(&self) -> u32 {
        todo!()
    }

    pub fn get_annotation(&self, _idx: u32) -> XTypesResult<()> {
        todo!()
    }

    pub fn add_member(&mut self, descriptor: MemberDescriptor) -> XTypesResult<()> {
        if let TK_ENUM | TK_BITMASK | TK_ANNOTATION | TK_STRUCTURE | TK_UNION | TK_BITSET =
            self.descriptor.kind
        {
        } else {
            return Err(XTypesError::IllegalOperation);
        }

        self.member_list.push(DynamicTypeMember { descriptor });

        Ok(())
    }

    pub fn apply_annotation(&mut self) -> XTypesResult<()> {
        todo!()
    }

    pub fn build(self) -> DynamicType {
        DynamicType {
            descriptor: Box::new(self.descriptor),
            member_list: self.member_list,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicType {
    descriptor: Box<TypeDescriptor>,
    member_list: Vec<DynamicTypeMember>,
}

impl DynamicType {
    pub fn get_descriptor(&self) -> &TypeDescriptor {
        &self.descriptor
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

pub struct DynamicDataFactory;

impl DynamicDataFactory {
    pub fn create_data(r#type: DynamicType) -> DynamicData {
        DynamicData {
            type_ref: r#type,
            abstract_data: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct DynamicData {
    type_ref: DynamicType,
    abstract_data: BTreeMap<MemberId, Box<dyn Any + Send + Sync>>,
}

impl DynamicData {
    pub fn type_ref(&self) -> &DynamicType {
        &self.type_ref
    }

    pub fn get_descriptor(&self, id: MemberId) -> XTypesResult<&MemberDescriptor> {
        self.type_ref
            .member_list
            .iter()
            .find(|m| m.get_id() == id)
            .map(|m| &m.descriptor)
            .ok_or(XTypesError::InvalidIndex)
    }

    pub fn set_descriptor(&mut self, _id: MemberId, _value: MemberDescriptor) -> XTypesResult<()> {
        todo!()
    }

    pub fn get_member_id_by_name(&self, name: &ObjectName) -> Option<MemberId> {
        self.type_ref
            .member_list
            .iter()
            .find(|m| m.get_name() == name)
            .map(|m| m.get_id())
    }

    pub fn get_member_id_at_index(&self, index: u32) -> Option<MemberId> {
        self.abstract_data.keys().nth(index as usize).cloned()
    }

    pub fn get_item_count(&self) -> u32 {
        match self.type_ref.get_kind() {
            TK_STRUCTURE => self.abstract_data.len() as u32,
            _ => todo!(),
        }
    }

    pub fn clear_all_values(&mut self) -> XTypesResult<()> {
        self.abstract_data.clear();
        Ok(())
    }

    pub fn clear_nonkey_values(&mut self) -> XTypesResult<()> {
        todo!()
    }

    pub fn clear_value(&mut self, id: MemberId) -> XTypesResult<()> {
        self.abstract_data
            .remove(&id)
            .ok_or(XTypesError::InvalidIndex)?;
        Ok(())
    }

    pub fn get_int32_value(&self, id: MemberId) -> XTypesResult<&i32> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int32_value(&mut self, id: MemberId, value: i32) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint32_value(&self, id: MemberId) -> XTypesResult<&u32> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint32_value(&mut self, id: MemberId, value: u32) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_int8_value(&self, id: MemberId) -> XTypesResult<&i8> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int8_value(&mut self, id: MemberId, value: i8) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint8_value(&self, id: MemberId) -> XTypesResult<&u8> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint8_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_int16_value(&self, id: MemberId) -> XTypesResult<&i16> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int16_value(&mut self, id: MemberId, value: i16) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint16_value(&self, id: MemberId) -> XTypesResult<&u16> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint16_value(&mut self, id: MemberId, value: u16) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_int64_value(&self, id: MemberId) -> XTypesResult<&i64> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int64_value(&mut self, id: MemberId, value: i64) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint64_value(&self, id: MemberId) -> XTypesResult<&u64> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint64_value(&mut self, id: MemberId, value: u64) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_float32_value(&self, id: MemberId) -> XTypesResult<&f32> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_float32_value(&mut self, id: MemberId, value: f32) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_float64_value(&self, id: MemberId) -> XTypesResult<&f64> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_float64_value(&mut self, id: MemberId, value: f64) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_char8_value(&self, id: MemberId) -> XTypesResult<&char> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_char8_value(&mut self, id: MemberId, value: char) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_byte_value(&self, id: MemberId) -> XTypesResult<&u8> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_byte_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_boolean_value(&self, id: MemberId) -> XTypesResult<&bool> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_boolean_value(&mut self, id: MemberId, value: bool) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_string_value(&self, id: MemberId) -> XTypesResult<&String> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_string_value(&mut self, id: MemberId, value: String) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_complex_value(&self, id: MemberId) -> XTypesResult<&DynamicData> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_complex_value(&mut self, id: MemberId, value: DynamicData) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_int32_values(&self, id: MemberId) -> XTypesResult<&Vec<i32>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int32_values(&mut self, id: MemberId, value: Vec<i32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint32_values(&self, id: MemberId) -> XTypesResult<&Vec<u32>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint32_values(&mut self, id: MemberId, value: Vec<u32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_int16_values(&self, id: MemberId) -> XTypesResult<&Vec<i16>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int16_values(&mut self, id: MemberId, value: Vec<i16>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint16_values(&self, id: MemberId) -> XTypesResult<&Vec<u16>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint16_values(&mut self, id: MemberId, value: Vec<u16>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_int64_values(&self, id: MemberId) -> XTypesResult<&Vec<i64>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int64_values(&mut self, id: MemberId, value: Vec<i64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_uint64_values(&self, id: MemberId) -> XTypesResult<&Vec<u64>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint64_values(&mut self, id: MemberId, value: Vec<u64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_float32_values(&self, id: MemberId) -> XTypesResult<&Vec<f32>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_float32_values(&mut self, id: MemberId, value: Vec<f32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_float64_values(&self, id: MemberId) -> XTypesResult<&Vec<f64>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_float64_values(&mut self, id: MemberId, value: Vec<f64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_char8_values(&self, id: MemberId) -> XTypesResult<&Vec<char>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_char8_values(&mut self, id: MemberId, value: Vec<char>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_byte_values(&self, id: MemberId) -> XTypesResult<&Vec<u8>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_byte_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_boolean_values(&self, id: MemberId) -> XTypesResult<&Vec<bool>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_boolean_values(&mut self, id: MemberId, value: Vec<bool>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_string_values(&self, id: MemberId) -> XTypesResult<&Vec<String>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_string_values(&mut self, id: MemberId, value: Vec<String>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    // Custom functions
    pub fn get_uint8_values(&self, id: MemberId) -> XTypesResult<&Vec<u8>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_uint8_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn get_complex_values(&self, id: MemberId) -> XTypesResult<&Vec<DynamicData>> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_complex_values(
        &mut self,
        id: MemberId,
        value: Vec<DynamicData>,
    ) -> XTypesResult<()> {
        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn insert_value<T: Any + Send + Sync>(mut self, id: MemberId, value: T) -> Self {
        self.abstract_data.insert(id, Box::new(value));
        self
    }

    pub fn remove_value<T: Any>(&mut self, id: MemberId) -> XTypesResult<T> {
        Ok(*self
            .abstract_data
            .remove(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast()
            .map_err(|_| XTypesError::InvalidType)?)
    }
}

pub trait DynamicDataInsert {
    fn get_dynamic_type() -> DynamicType;
}

impl DynamicDataInsert for u8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT8)
    }
}
impl DynamicDataInsert for i8 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT8)
    }
}

impl DynamicDataInsert for u16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT16)
    }
}

impl DynamicDataInsert for i16 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT16)
    }
}

impl DynamicDataInsert for u32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT32)
    }
}

impl DynamicDataInsert for i64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT64)
    }
}

impl DynamicDataInsert for u64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_UINT64)
    }
}

impl DynamicDataInsert for i32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_INT32)
    }
}

impl DynamicDataInsert for String {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_string_type(u32::MAX).build()
    }
}

impl DynamicDataInsert for bool {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_BOOLEAN)
    }
}

impl DynamicDataInsert for f32 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_FLOAT32)
    }
}

impl DynamicDataInsert for f64 {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_FLOAT64)
    }
}

impl DynamicDataInsert for char {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::get_primitive_type(TK_CHAR8)
    }
}

impl<const N: usize> DynamicDataInsert for [u8; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(u8::get_dynamic_type(), vec![N as u32]).build()
    }
}

impl DynamicDataInsert for &'_ [u8] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }
}

impl DynamicDataInsert for Vec<u8> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u8::get_dynamic_type(), u32::MAX).build()
    }
}

impl DynamicDataInsert for Vec<u16> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(u16::get_dynamic_type(), u32::MAX).build()
    }
}

impl DynamicDataInsert for Vec<String> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(String::get_dynamic_type(), u32::MAX)
            .build()
    }
}

impl<T: TypeSupport> DynamicDataInsert for T {
    fn get_dynamic_type() -> DynamicType {
        T::get_type()
    }
}

impl<T: TypeSupport, const N: usize> DynamicDataInsert for [T; N] {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_array_type(T::get_type(), vec![N as u32]).build()
    }
}

impl<T: TypeSupport> DynamicDataInsert for Vec<T> {
    fn get_dynamic_type() -> DynamicType {
        DynamicTypeBuilderFactory::create_sequence_type(T::get_type(), u32::MAX).build()
    }
}
