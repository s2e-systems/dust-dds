use crate::xtypes::{binding::DataKind, error::XTypesResult, type_object::TypeObject};

use super::error::XTypesError;
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec, vec::Vec};

pub type BoundSeq = Vec<u32>;
pub type IncludePathSeq = Vec<String>;
pub type ObjectName = String;

// ---------- TypeKinds (begin) -------------------
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum TypeKind {
    // Primitive TKs
    NONE = 0x00,
    BOOLEAN = 0x01,
    BYTE = 0x02,
    INT16 = 0x03,
    INT32 = 0x04,
    INT64 = 0x05,
    UINT16 = 0x06,
    UINT32 = 0x07,
    UINT64 = 0x08,
    FLOAT32 = 0x09,
    FLOAT64 = 0x0A,
    FLOAT128 = 0x0B,
    INT8 = 0x0C,
    UINT8 = 0x0D,
    CHAR8 = 0x10,
    CHAR16 = 0x11,
    // String TK;
    STRING8 = 0x20,
    STRING16 = 0x21,
    // Constructed/Named type;
    ALIAS = 0x30,
    // Enumerated TK;
    ENUM = 0x40,
    BITMASK = 0x41,
    // Structured TK;
    ANNOTATION = 0x50,
    STRUCTURE = 0x51,
    UNION = 0x52,
    BITSET = 0x53,
    // Collection TK;
    SEQUENCE = 0x60,
    ARRAY = 0x61,
    MAP = 0x62,
}

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
                kind: TypeKind::STRING8,
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
                kind: TypeKind::SEQUENCE,
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
                kind: TypeKind::ARRAY,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtensibilityKind {
    Final,
    Appendable,
    Mutable,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TryConstructKind {
    UseDefault,
    Discard,
    Trim,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct MemberDescriptor {
    pub name: ObjectName,
    pub id: MemberId,
    pub r#type: DynamicType,
    pub default_value: Option<DataKind>,
    pub index: u32,
    pub label: UnionCaseLabelSeq,
    pub try_construct_kind: TryConstructKind,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
    pub is_shared: bool,
    pub is_default_label: bool,
}

#[derive(Debug, Clone, PartialEq)]
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
        if let TypeKind::ENUM
        | TypeKind::BITMASK
        | TypeKind::ANNOTATION
        | TypeKind::STRUCTURE
        | TypeKind::UNION
        | TypeKind::BITSET = self.descriptor.kind
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicData {
    type_ref: DynamicType,
    abstract_data: BTreeMap<MemberId, DataKind>,
}

impl DynamicData {
    pub fn type_ref(&self) -> &DynamicType {
        &self.type_ref
    }

    pub(crate) fn make_descriptor_extensibility_kind_final(&mut self) {
        self.type_ref.descriptor.extensibility_kind = ExtensibilityKind::Final
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

    pub fn get_member_id_at_index(&self, index: u32) -> XTypesResult<MemberId> {
        self.abstract_data
            .keys()
            .nth(index as usize)
            .cloned()
            .ok_or(XTypesError::InvalidIndex)
    }

    pub fn get_item_count(&self) -> u32 {
        match self.type_ref.get_kind() {
            TypeKind::STRUCTURE => self.abstract_data.len() as u32,
            _ => todo!(),
        }
    }

    pub fn clear_all_values(&mut self) -> XTypesResult<()> {
        self.abstract_data.clear();
        Ok(())
    }

    pub fn clear_nonkey_values(&mut self) -> XTypesResult<()> {
        for index in 0..self.type_ref.get_member_count() {
            let member = self.type_ref.get_member_by_index(index)?;
            if !member.get_descriptor()?.is_key {
                let member_id = member.get_id();
                self.abstract_data.remove(&member_id);
            }
        }
        Ok(())
    }

    pub fn clear_value(&mut self, id: MemberId) -> XTypesResult<()> {
        self.abstract_data
            .remove(&id)
            .ok_or(XTypesError::InvalidIndex)?;
        Ok(())
    }

    pub fn get_int32_value(&self, id: MemberId) -> XTypesResult<&i32> {
        if let DataKind::Int32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int32_value(mut self, id: MemberId, value: i32) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Int32(value));
        Ok(self)
    }

    pub fn get_uint32_value(&self, id: MemberId) -> XTypesResult<&u32> {
        if let DataKind::UInt32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint32_value(mut self, id: MemberId, value: u32) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::UInt32(value));
        Ok(self)
    }

    pub fn get_int8_value(&self, id: MemberId) -> XTypesResult<&i8> {
        if let DataKind::Int8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int8_value(&mut self, id: MemberId, value: i8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Int8(value));
        Ok(())
    }

    pub fn get_uint8_value(&self, id: MemberId) -> XTypesResult<&u8> {
        if let DataKind::UInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint8_value(mut self, id: MemberId, value: u8) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::UInt8(value));
        Ok(self)
    }

    pub fn get_int16_value(&self, id: MemberId) -> XTypesResult<&i16> {
        if let DataKind::Int16(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int16_value(mut self, id: MemberId, value: i16) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Int16(value));
        Ok(self)
    }

    pub fn get_uint16_value(&self, id: MemberId) -> XTypesResult<&u16> {
        if let DataKind::UInt16(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint16_value(mut self, id: MemberId, value: u16) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::UInt16(value));
        Ok(self)
    }

    pub fn get_int64_value(&self, id: MemberId) -> XTypesResult<&i64> {
        if let DataKind::Int64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int64_value(mut self, id: MemberId, value: i64) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Int64(value));
        Ok(self)
    }

    pub fn get_uint64_value(&self, id: MemberId) -> XTypesResult<&u64> {
        if let DataKind::UInt64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint64_value(mut self, id: MemberId, value: u64) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::UInt64(value));
        Ok(self)
    }

    pub fn get_float32_value(&self, id: MemberId) -> XTypesResult<&f32> {
        if let DataKind::Float32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float32_value(mut self, id: MemberId, value: f32) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Float32(value));
        Ok(self)
    }

    pub fn get_float64_value(&self, id: MemberId) -> XTypesResult<&f64> {
        if let DataKind::Float64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float64_value(mut self, id: MemberId, value: f64) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Float64(value));
        Ok(self)
    }

    pub fn get_char8_value(&self, id: MemberId) -> XTypesResult<&char> {
        if let DataKind::Char8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_char8_value(mut self, id: MemberId, value: char) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Char8(value));
        Ok(self)
    }

    pub fn get_byte_value(&self, id: MemberId) -> XTypesResult<&u8> {
        if let DataKind::UInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_byte_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::UInt8(value));
        Ok(())
    }

    pub fn get_boolean_value(&self, id: MemberId) -> XTypesResult<&bool> {
        if let DataKind::Boolean(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_boolean_value(mut self, id: MemberId, value: bool) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::Boolean(value));
        Ok(self)
    }

    pub fn get_string_value(&self, id: MemberId) -> XTypesResult<&String> {
        if let DataKind::String(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_string_value(mut self, id: MemberId, value: String) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::String(value));
        Ok(self)
    }

    pub fn get_complex_value(&self, id: MemberId) -> XTypesResult<&DynamicData> {
        if let DataKind::ComplexValue(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn get_data_kind(&self, id: MemberId) -> XTypesResult<&DataKind> {
        self.abstract_data.get(&id).ok_or(XTypesError::InvalidIndex)
    }

    pub fn set_complex_value(&mut self, id: MemberId, value: DynamicData) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::ComplexValue(value));
        Ok(())
    }

    pub fn get_int32_values(&self, id: MemberId) -> XTypesResult<&Vec<i32>> {
        if let DataKind::Int32List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int32_values(&mut self, id: MemberId, value: Vec<i32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Int32List(value));
        Ok(())
    }

    pub fn get_uint32_values(&self, id: MemberId) -> XTypesResult<&Vec<u32>> {
        if let DataKind::UInt32List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint32_values(&mut self, id: MemberId, value: Vec<u32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::UInt32List(value));
        Ok(())
    }

    pub fn get_int16_values(&self, id: MemberId) -> XTypesResult<&Vec<i16>> {
        if let DataKind::Int16List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int16_values(&mut self, id: MemberId, value: Vec<i16>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Int16List(value));
        Ok(())
    }

    pub fn get_uint16_values(&self, id: MemberId) -> XTypesResult<&Vec<u16>> {
        if let DataKind::UInt16List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint16_values(&mut self, id: MemberId, value: Vec<u16>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::UInt16List(value));
        Ok(())
    }

    pub fn get_int64_values(&self, id: MemberId) -> XTypesResult<&Vec<i64>> {
        if let DataKind::Int64List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int64_values(&mut self, id: MemberId, value: Vec<i64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Int64List(value));
        Ok(())
    }

    pub fn get_uint64_values(&self, id: MemberId) -> XTypesResult<&Vec<u64>> {
        if let DataKind::UInt64List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint64_values(&mut self, id: MemberId, value: Vec<u64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::UInt64List(value));
        Ok(())
    }

    pub fn get_float32_values(&self, id: MemberId) -> XTypesResult<&Vec<f32>> {
        if let DataKind::Float32List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float32_values(&mut self, id: MemberId, value: Vec<f32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Float32List(value));
        Ok(())
    }

    pub fn get_float64_values(&self, id: MemberId) -> XTypesResult<&Vec<f64>> {
        if let DataKind::Float64List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float64_values(&mut self, id: MemberId, value: Vec<f64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Float64List(value));
        Ok(())
    }

    pub fn get_char8_values(&self, id: MemberId) -> XTypesResult<&Vec<char>> {
        if let DataKind::Char8List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_char8_values(&mut self, id: MemberId, value: Vec<char>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Char8List(value));
        Ok(())
    }

    pub fn get_byte_values(&self, id: MemberId) -> XTypesResult<&Vec<u8>> {
        if let DataKind::UInt8List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_byte_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::UInt8List(value));
        Ok(())
    }

    pub fn get_boolean_values(&self, id: MemberId) -> XTypesResult<&Vec<bool>> {
        if let DataKind::BooleanList(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_boolean_values(&mut self, id: MemberId, value: Vec<bool>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::BooleanList(value));
        Ok(())
    }

    pub fn get_string_values(&self, id: MemberId) -> XTypesResult<&Vec<String>> {
        if let DataKind::StringList(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_string_values(mut self, id: MemberId, value: Vec<String>) -> XTypesResult<Self> {
        self.abstract_data.insert(id, DataKind::StringList(value));
        Ok(self)
    }

    // Custom functions
    pub fn get_uint8_values(&self, id: MemberId) -> XTypesResult<&Vec<u8>> {
        if let DataKind::UInt8List(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint8_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::UInt8List(value));
        Ok(())
    }

    pub fn set_int8_values(&mut self, id: MemberId, value: Vec<i8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataKind::Int8List(value));
        Ok(())
    }

    pub fn get_complex_values(&self, id: MemberId) -> XTypesResult<&Vec<DynamicData>> {
        if let DataKind::ComplexValueList(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidIndex)?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_complex_values(
        &mut self,
        id: MemberId,
        value: Vec<DynamicData>,
    ) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataKind::ComplexValueList(value));
        Ok(())
    }

    pub fn set_value<T: Into<DataKind>>(mut self, id: MemberId, value: T) -> Self {
        self.abstract_data.insert(id, value.into());
        self
    }

    pub fn get_value(&self, id: MemberId) -> XTypesResult<&DataKind> {
        self.abstract_data.get(&id).ok_or(XTypesError::InvalidIndex)
    }
}
