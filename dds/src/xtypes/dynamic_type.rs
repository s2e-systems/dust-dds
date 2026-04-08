use super::error::XTypesError;
use crate::xtypes::{
    data_storage::{DataStorage, DataStorageMapping},
    error::XTypesResult,
    type_object::TypeObject,
};
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

pub type BoundSeq = Option<u32>;
pub type IncludePathSeq = Vec<String>;
pub type ObjectName<'a> = &'a str;

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
    pub fn get_primitive_type(kind: TypeKind) -> impl DynamicType {
        StaticTypeInformation {
            descriptor: Box::leak(Box::new(TypeDescriptor {
                kind,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: None,
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            })),
            member_list: &[],
        }
    }

    pub fn create_type(descriptor: TypeDescriptor) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor,
            member_list: Vec::new(),
        }
    }

    pub fn create_type_copy(r#_type: impl DynamicType) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_type_w_type_object(_type_object: TypeObject) -> DynamicTypeBuilder {
        todo!()
    }

    pub fn create_string_type(bound: u32) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TypeKind::STRING8,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: Some(bound),
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

    pub fn create_sequence_type(
        element_type: &'static dyn DynamicType,
        bound: u32,
    ) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TypeKind::SEQUENCE,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: Some(bound),
                element_type: Some(element_type),
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            member_list: Vec::new(),
        }
    }

    pub fn create_array_type(
        element_type: &'static dyn DynamicType,
        bound: BoundSeq,
    ) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TypeKind::ARRAY,
                name: "",
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
        _key_element_type: &'static dyn DynamicType,
        _element_type: &'static dyn DynamicType,
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

pub type Parameters = BTreeMap<ObjectName<'static>, ObjectName<'static>>;

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

pub struct TypeDescriptor {
    pub kind: TypeKind,
    pub name: ObjectName<'static>,
    pub base_type: Option<&'static dyn DynamicType>,
    pub discriminator_type: Option<&'static dyn DynamicType>,
    pub bound: BoundSeq,
    pub element_type: Option<&'static dyn DynamicType>,
    pub key_element_type: Option<&'static dyn DynamicType>,
    pub extensibility_kind: ExtensibilityKind,
    pub is_nested: bool,
}

pub type MemberId = u32;
pub type UnionCaseLabelSeq = Option<i32>;

pub struct MemberDescriptor {
    pub name: ObjectName<'static>,
    pub id: MemberId,
    pub r#type: &'static dyn DynamicType,
    pub default_value: Option<&'static str>,
    pub index: u32,
    pub label: UnionCaseLabelSeq,
    pub try_construct_kind: TryConstructKind,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
    pub is_shared: bool,
    pub is_default_label: bool,
}

pub struct DynamicTypeMember {
    pub descriptor: MemberDescriptor,
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
    pub fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
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

    pub fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
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
    ) -> Result<Vec<(ObjectName<'static>, DynamicTypeMember)>, XTypesError> {
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

    pub fn build(self) -> &'static dyn DynamicType {
        Box::leak(Box::new(StaticTypeInformation {
            descriptor: Box::leak(Box::new(self.descriptor)),
            member_list: Vec::leak(self.member_list),
        }))
    }
}

pub trait DynamicType: Send + Sync {
    fn get_descriptor(&self) -> &TypeDescriptor;

    fn get_name(&self) -> ObjectName<'static>;

    fn get_kind(&self) -> TypeKind;

    fn get_member_by_name(&self, name: ObjectName) -> Result<&DynamicTypeMember, XTypesError>;
    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    fn get_member(&self, id: MemberId) -> Result<&DynamicTypeMember, XTypesError>;
    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);

    fn get_member_count(&self) -> u32;
    fn get_member_by_index(&self, index: u32) -> Result<&DynamicTypeMember, XTypesError>;

    // fn get_annotation_count(&self) -> u32;
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);
}

pub struct StaticTypeInformation {
    pub descriptor: &'static TypeDescriptor,
    pub member_list: &'static [DynamicTypeMember],
}

impl DynamicType for StaticTypeInformation {
    fn get_descriptor(&self) -> &TypeDescriptor {
        self.descriptor
    }
    fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
    }
    fn get_kind(&self) -> TypeKind {
        self.descriptor.kind
    }

    fn get_member_by_name(&self, name: ObjectName) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .iter()
            .find(|m| m.get_name() == name)
            .ok_or(XTypesError::InvalidName)
    }

    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    fn get_member(&self, id: MemberId) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .iter()
            .find(|m| m.get_id() == id)
            .ok_or(XTypesError::InvalidId(id))
    }

    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);

    fn get_member_count(&self) -> u32 {
        self.member_list.len() as u32
    }
    fn get_member_by_index(&self, index: u32) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .get(index as usize)
            .ok_or(XTypesError::InvalidIndex(index))
    }

    // fn get_annotation_count(&self) -> u32;
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);
}

pub struct DynamicDataFactory;

impl DynamicDataFactory {
    pub fn create_data() -> DynamicData {
        DynamicData {
            abstract_data: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicData {
    abstract_data: BTreeMap<MemberId, DataStorage>,
}

impl DynamicData {
    pub fn get_descriptor<'a>(
        &self,
        type_ref: &'a dyn DynamicType,
        id: MemberId,
    ) -> XTypesResult<&'a MemberDescriptor> {
        Ok(&type_ref.get_member(id)?.descriptor)
    }

    pub fn set_descriptor(&mut self, _id: MemberId, _value: MemberDescriptor) -> XTypesResult<()> {
        todo!()
    }

    pub fn get_member_id_by_name(
        &self,
        type_ref: &dyn DynamicType,
        name: &str,
    ) -> Option<MemberId> {
        type_ref.get_member_by_name(name).ok().map(|m| m.get_id())
    }

    pub fn get_member_id_at_index(&self, index: u32) -> XTypesResult<MemberId> {
        self.abstract_data
            .keys()
            .nth(index as usize)
            .cloned()
            .ok_or(XTypesError::InvalidIndex(index))
    }

    pub fn get_item_count(&self) -> u32 {
        self.abstract_data.len() as u32
    }

    pub fn clear_all_values(&mut self) -> XTypesResult<()> {
        self.abstract_data.clear();
        Ok(())
    }

    pub fn clear_nonkey_values(&mut self, type_ref: &dyn DynamicType) -> XTypesResult<()> {
        for index in 0..type_ref.get_member_count() {
            let member = type_ref.get_member_by_index(index)?;
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
            .ok_or(XTypesError::InvalidId(id))?;
        Ok(())
    }

    pub fn get_int32_value(&self, id: MemberId) -> XTypesResult<&i32> {
        if let DataStorage::Int32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int32_value(&mut self, id: MemberId, value: i32) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int32(value));
        Ok(())
    }

    pub fn get_uint32_value(&self, id: MemberId) -> XTypesResult<&u32> {
        if let DataStorage::UInt32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint32_value(&mut self, id: MemberId, value: u32) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt32(value));
        Ok(())
    }

    pub fn get_int8_value(&self, id: MemberId) -> XTypesResult<&i8> {
        if let DataStorage::Int8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int8_value(&mut self, id: MemberId, value: i8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int8(value));
        Ok(())
    }

    pub fn get_uint8_value(&self, id: MemberId) -> XTypesResult<&u8> {
        if let DataStorage::UInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint8_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt8(value));
        Ok(())
    }

    pub fn get_int16_value(&self, id: MemberId) -> XTypesResult<&i16> {
        if let DataStorage::Int16(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int16_value(&mut self, id: MemberId, value: i16) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int16(value));
        Ok(())
    }

    pub fn get_uint16_value(&self, id: MemberId) -> XTypesResult<&u16> {
        if let DataStorage::UInt16(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint16_value(&mut self, id: MemberId, value: u16) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt16(value));
        Ok(())
    }

    pub fn get_int64_value(&self, id: MemberId) -> XTypesResult<&i64> {
        if let DataStorage::Int64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int64_value(&mut self, id: MemberId, value: i64) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int64(value));
        Ok(())
    }

    pub fn get_uint64_value(&self, id: MemberId) -> XTypesResult<&u64> {
        if let DataStorage::UInt64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint64_value(&mut self, id: MemberId, value: u64) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt64(value));
        Ok(())
    }

    pub fn get_float32_value(&self, id: MemberId) -> XTypesResult<&f32> {
        if let DataStorage::Float32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float32_value(&mut self, id: MemberId, value: f32) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Float32(value));
        Ok(())
    }

    pub fn get_float64_value(&self, id: MemberId) -> XTypesResult<&f64> {
        if let DataStorage::Float64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float64_value(&mut self, id: MemberId, value: f64) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Float64(value));
        Ok(())
    }

    pub fn get_char8_value(&self, id: MemberId) -> XTypesResult<&char> {
        if let DataStorage::Char8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_char8_value(&mut self, id: MemberId, value: char) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Char8(value));
        Ok(())
    }

    pub fn get_byte_value(&self, id: MemberId) -> XTypesResult<&u8> {
        if let DataStorage::UInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_byte_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt8(value));
        Ok(())
    }

    pub fn get_boolean_value(&self, id: MemberId) -> XTypesResult<&bool> {
        if let DataStorage::Boolean(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_boolean_value(&mut self, id: MemberId, value: bool) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_string_value(&self, id: MemberId) -> XTypesResult<&String> {
        if let DataStorage::String(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_string_value(&mut self, id: MemberId, value: String) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::String(value));
        Ok(())
    }

    pub fn get_complex_value(&self, id: MemberId) -> XTypesResult<&DynamicData> {
        if let DataStorage::ComplexValue(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn get_data_kind(&self, id: MemberId) -> XTypesResult<&DataStorage> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))
    }

    pub fn set_complex_value(&mut self, id: MemberId, value: DynamicData) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::ComplexValue(value));
        Ok(())
    }

    pub fn get_int32_values(&self, id: MemberId) -> XTypesResult<&[i32]> {
        if let DataStorage::SequenceInt32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int32_values(&mut self, id: MemberId, value: Vec<i32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_uint32_values(&self, id: MemberId) -> XTypesResult<&[u32]> {
        if let DataStorage::SequenceUInt32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint32_values(&mut self, id: MemberId, value: Vec<u32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_int16_values(&self, id: MemberId) -> XTypesResult<&[i16]> {
        if let DataStorage::SequenceInt16(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int16_values(&mut self, id: MemberId, value: Vec<i16>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_uint16_values(&self, id: MemberId) -> XTypesResult<&[u16]> {
        if let DataStorage::SequenceUInt16(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint16_values(&mut self, id: MemberId, value: Vec<u16>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_int64_values(&self, id: MemberId) -> XTypesResult<&[i64]> {
        if let DataStorage::SequenceInt64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_int64_values(&mut self, id: MemberId, value: Vec<i64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_uint64_values(&self, id: MemberId) -> XTypesResult<&[u64]> {
        if let DataStorage::SequenceUInt64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint64_values(&mut self, id: MemberId, value: Vec<u64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_float32_values(&self, id: MemberId) -> XTypesResult<&[f32]> {
        if let DataStorage::SequenceFloat32(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float32_values(&mut self, id: MemberId, value: Vec<f32>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_float64_values(&self, id: MemberId) -> XTypesResult<&[f64]> {
        if let DataStorage::SequenceFloat64(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float64_values(&mut self, id: MemberId, value: Vec<f64>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_char8_values(&self, id: MemberId) -> XTypesResult<&[char]> {
        if let DataStorage::SequenceChar8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_char8_values(&mut self, id: MemberId, value: Vec<char>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_byte_values(&self, id: MemberId) -> XTypesResult<&[u8]> {
        if let DataStorage::SequenceUInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_byte_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_boolean_values(&self, id: MemberId) -> XTypesResult<&[bool]> {
        if let DataStorage::SequenceBoolean(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_boolean_values(&mut self, id: MemberId, value: Vec<bool>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_string_values(&self, id: MemberId) -> XTypesResult<&[String]> {
        if let DataStorage::SequenceString(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_string_values(&mut self, id: MemberId, value: Vec<String>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    // Custom functions
    pub fn get_uint8_values(&self, id: MemberId) -> XTypesResult<&[u8]> {
        if let DataStorage::SequenceUInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn get_int8_values(&self, id: MemberId) -> XTypesResult<&[i8]> {
        if let DataStorage::SequenceInt8(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_uint8_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn set_int8_values(&mut self, id: MemberId, value: Vec<i8>) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    pub fn get_complex_values(&self, id: MemberId) -> XTypesResult<&[DynamicData]> {
        if let DataStorage::SequenceComplexValue(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
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
            .insert(id, DataStorage::SequenceComplexValue(value));
        Ok(())
    }

    pub fn set_value(&mut self, id: MemberId, value: DataStorage) {
        self.abstract_data.insert(id, value);
    }

    pub fn get_value(&self, id: MemberId) -> XTypesResult<&DataStorage> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))
    }

    pub fn remove_value(&mut self, id: MemberId) -> XTypesResult<DataStorage> {
        self.abstract_data
            .remove(&id)
            .ok_or(XTypesError::InvalidId(id))
    }
}
