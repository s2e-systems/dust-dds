use core::any::Any;

use crate::xtypes::{error::XTypesResult, type_object::TypeObject};

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

    pub fn create_type_copy(r#type: DynamicType) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_type_object(type_object: TypeObject) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_string_type(bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_wstring_type(_bound: u32) -> DynamicTypeBuilder {
        unimplemented!("wstring not supported in Rust")
    }

    pub fn create_sequence_type(element_type: DynamicType, bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_array_type(element_type: DynamicType, bound: BoundSeq) -> DynamicTypeBuilder {
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

    pub fn get_member(&self, id: MemberId) -> Result<DynamicTypeMember, XTypesError> {
        todo!()
    }

    pub fn get_all_members(&self) -> Result<Vec<(MemberId, DynamicTypeMember)>, XTypesError> {
        todo!()
    }

    pub fn get_annotation_count(&self) -> u32 {
        todo!()
    }

    pub fn get_annotation(&self, idx: u32) -> XTypesResult<()> {
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

pub struct DynamicDataFactory;

impl DynamicDataFactory {
    pub fn create_data(r#type: DynamicType) -> DynamicData {
        DynamicData {
            type_ref: r#type,
            abstract_data: BTreeMap::new(),
        }
    }
}

pub struct DynamicData {
    type_ref: DynamicType,
    abstract_data: BTreeMap<MemberId, Box<dyn Any>>,
}

impl DynamicData {
    pub fn get_descriptor(&self, id: MemberId) -> XTypesResult<MemberDescriptor> {
        todo!()
    }

    pub fn set_descriptor(&mut self, id: MemberId, value: MemberDescriptor) -> XTypesResult<()> {
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
        self.type_ref
            .member_list
            .get(index as usize)
            .map(|m| m.get_id())
    }

    // unsigned long get_item_count();

    pub fn clear_all_values(&mut self) -> XTypesResult<()> {
        self.abstract_data.clear();
        Ok(())
    }

    pub fn clear_nonkey_values(&mut self) -> XTypesResult<()> {
        todo!()
    }

    pub fn clear_value(&mut self, id: MemberId) -> XTypesResult<()> {
        todo!()
    }

    pub fn get_int32_value(&self, id: MemberId) -> XTypesResult<&i32> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
            .downcast_ref()
            .ok_or(XTypesError::InvalidType)
    }

    pub fn set_int32_value(&mut self, id: MemberId, value: i32) -> XTypesResult<()> {
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkInt32Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkUint32Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkInt8Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkUint8Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkInt16Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     DynamicTypeBuilderFactory::ge ::TkUint16Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkInt64Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkUint64Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkFloat32Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkFloat64Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkChar8Type
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkByteType
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::TkByteType
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiString8Small { string_sdefn } => {
        //         if value.len() > string_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //     }
        //     TypeIdentifier::TiString8Large { string_ldefn } => {
        //         if value.len() > string_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // if !matches!(
        //     self.type_ref
        //         .get_member_by_index(id)?
        //         .get_descriptor()?
        //         .r#type,
        //     TypeIdentifier::EkComplete { complete: _ }
        // ) {
        //     return Err(XTypesError::InvalidType);
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkInt32Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkInt32Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkUint32Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkUint32Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkInt16Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkInt16Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkUint16Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkUint16Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkInt64Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkInt64Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkUint64Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkUint64Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkFloat32Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkFloat32Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkFloat64Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkFloat64Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkChar8Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkChar8Type) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkByteType) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkByteType) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_sdefn.element_identifier, TypeIdentifier::TkBoolean) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(seq_ldefn.element_identifier, TypeIdentifier::TkBoolean) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

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
        // match &self
        //     .type_ref
        //     .get_member_by_index(id)?
        //     .get_descriptor()?
        //     .r#type
        // {
        //     TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
        //         if value.len() > seq_sdefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(
        //             seq_sdefn.element_identifier,
        //             TypeIdentifier::TiString8Small { string_sdefn: _ }
        //         ) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
        //         if value.len() > seq_ldefn.bound as usize {
        //             return Err(XTypesError::InvalidData);
        //         }
        //         if !matches!(
        //             seq_ldefn.element_identifier,
        //             TypeIdentifier::TiString8Small { string_sdefn: _ }
        //         ) {
        //             return Err(XTypesError::InvalidType);
        //         }
        //     }
        //     _ => return Err(XTypesError::InvalidType),
        // }

        self.abstract_data.insert(id, Box::new(value));
        Ok(())
    }

    pub fn insert_value<T: Any>(mut self, id: MemberId, value: T) -> Self {
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
