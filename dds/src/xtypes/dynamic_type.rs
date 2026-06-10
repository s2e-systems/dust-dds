use super::error::XTypesError;
use crate::{
    infrastructure::type_support::TypeSupport,
    xtypes::{
        data_storage::{DataStorage, DataStorageMapping},
        error::XTypesResult,
        type_object::TypeObject,
    },
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
    pub fn get_primitive_type(kind: TypeKind) -> DynamicType {
        DynamicType {
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

    pub fn create_sequence_type(element_type: DynamicType, bound: u32) -> DynamicTypeBuilder {
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

    pub fn create_array_type(element_type: DynamicType, bound: BoundSeq) -> DynamicTypeBuilder {
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

    #[cfg(feature = "xtypes-xml")]
    pub fn create_type_w_document(
        document: &str,
        type_name: &str,
        _include_paths: Vec<String>,
    ) -> XTypesResult<DynamicTypeBuilder> {
        let doc = roxmltree::Document::parse(document).map_err(|_| XTypesError::InvalidData)?;

        let path: Vec<&str> = type_name.split("::").collect();
        let (struct_name, module_path) = path.split_last().ok_or(XTypesError::InvalidData)?;

        let mut target_node = None;
        for node in doc.descendants() {
            if node.is_element()
                && (node.tag_name().name() == "struct"
                    || node.tag_name().name() == "union"
                    || node.tag_name().name() == "enum")
            {
                if let Some(name) = node.attribute("name") {
                    if &name == struct_name {
                        let mut current_node = node.parent();
                        let mut matches = true;
                        for &expected_mod in module_path.iter().rev() {
                            let mut found = false;
                            while let Some(parent) = current_node {
                                current_node = parent.parent();
                                if parent.is_element() && parent.tag_name().name() == "module" {
                                    if parent.attribute("name") == Some(expected_mod) {
                                        found = true;
                                    }
                                    break;
                                }
                            }
                            if !found {
                                matches = false;
                                break;
                            }
                        }
                        if matches {
                            target_node = Some(node);
                            break;
                        }
                    }
                }
            }
        }

        let target_node = target_node.ok_or(XTypesError::InvalidData)?;
        let is_union = target_node.tag_name().name() == "union";
        let is_enum = target_node.tag_name().name() == "enum";

        let ext_str = target_node.attribute("extensibility").unwrap_or("final");
        let extensibility_kind = match ext_str {
            "final" => ExtensibilityKind::Final,
            "appendable" => ExtensibilityKind::Appendable,
            "mutable" => ExtensibilityKind::Mutable,
            _ => ExtensibilityKind::Final,
        };

        let parse_type_kind = |m_type: &str| -> XTypesResult<TypeKind> {
            match m_type {
                "boolean" => Ok(TypeKind::BOOLEAN),
                "byte" => Ok(TypeKind::BYTE),
                "char8" => Ok(TypeKind::CHAR8),
                "char16" => Ok(TypeKind::CHAR16),
                "int32" => Ok(TypeKind::INT32),
                "uint32" => Ok(TypeKind::UINT32),
                "int8" => Ok(TypeKind::INT8),
                "uint8" => Ok(TypeKind::UINT8),
                "int16" => Ok(TypeKind::INT16),
                "uint16" => Ok(TypeKind::UINT16),
                "int64" => Ok(TypeKind::INT64),
                "uint64" => Ok(TypeKind::UINT64),
                "float32" => Ok(TypeKind::FLOAT32),
                "float64" => Ok(TypeKind::FLOAT64),
                "float128" => Ok(TypeKind::FLOAT128),
                "string" => Ok(TypeKind::STRING8),
                "wstring" => Ok(TypeKind::STRING16),
                _ => Err(XTypesError::InvalidData),
            }
        };

        let resolve_member_type = |m_type: &str,
                                   non_basic_type_name: Option<&str>,
                                   string_max_length: Option<&str>,
                                   array_dimensions: Option<&str>|
         -> XTypesResult<DynamicType> {
            let mut type_ptr: DynamicType = if m_type == "nonBasic" {
                let non_basic_name = non_basic_type_name.ok_or(XTypesError::InvalidData)?;
                let mut full_name = module_path.join("::");
                if !full_name.is_empty() {
                    full_name.push_str("::");
                }
                full_name.push_str(non_basic_name);
                let nested_builder =
                    Self::create_type_w_document(document, &full_name, _include_paths.clone())?;
                nested_builder.build()
            } else if m_type == "string" {
                let bound = string_max_length.unwrap_or("0").parse().unwrap_or(0);
                let builder = Self::create_string_type(bound);
                builder.build()
            } else if m_type == "wstring" {
                let bound = string_max_length.unwrap_or("0").parse().unwrap_or(0);
                let builder = Self::create_wstring_type(bound);
                builder.build()
            } else {
                let type_kind = parse_type_kind(m_type)?;
                Self::get_primitive_type(type_kind)
            };

            if let Some(dimensions) = array_dimensions {
                let dims: Vec<u32> = dimensions
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                for dim in dims.into_iter().rev() {
                    let builder = Self::create_array_type(type_ptr, Some(dim));
                    type_ptr = builder.build();
                }
            }
            Ok(type_ptr)
        };

        let mut discriminator_type = None;
        if is_union {
            for child in target_node.children() {
                if child.is_element() && child.tag_name().name() == "discriminator" {
                    let d_type = child.attribute("type").ok_or(XTypesError::InvalidData)?;
                    let non_basic_type_name = child.attribute("nonBasicTypeName");
                    let string_max_length = child.attribute("stringMaxLength");
                    let type_ptr =
                        resolve_member_type(d_type, non_basic_type_name, string_max_length, None)?;
                    discriminator_type = Some(type_ptr);
                    break;
                }
            }
        }

        let name: &'static str = Box::leak(type_name.to_string().into_boxed_str());
        let descriptor = TypeDescriptor {
            kind: if is_union {
                TypeKind::UNION
            } else if is_enum {
                TypeKind::ENUM
            } else {
                TypeKind::STRUCTURE
            },
            name,
            base_type: None,
            discriminator_type,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind,
            is_nested: target_node.attribute("nested") == Some("true"),
        };

        let mut builder = Self::create_type(descriptor);

        let mut member_id = 0;
        if is_union {
            for child in target_node.children() {
                if child.is_element() && child.tag_name().name() == "case" {
                    let mut m_name = None;
                    let mut m_type = None;
                    let mut non_basic_type_name = None;
                    let mut string_max_length = None;
                    let mut array_dimensions = None;
                    let mut label = None;
                    let mut is_default_label = false;

                    for case_child in child.children() {
                        if case_child.is_element()
                            && case_child.tag_name().name() == "caseDiscriminator"
                        {
                            let value_str = case_child
                                .attribute("value")
                                .ok_or(XTypesError::InvalidData)?;
                            if value_str == "default" {
                                is_default_label = true;
                            } else if let Some(hex) = value_str.strip_prefix("0x") {
                                label = Some(
                                    i32::from_str_radix(hex, 16)
                                        .map_err(|_| XTypesError::InvalidData)?,
                                );
                            } else {
                                label = Some(
                                    value_str
                                        .parse::<i32>()
                                        .map_err(|_| XTypesError::InvalidData)?,
                                );
                            }
                        } else if case_child.is_element()
                            && case_child.tag_name().name() == "member"
                        {
                            m_name = case_child.attribute("name");
                            m_type = case_child.attribute("type");
                            non_basic_type_name = case_child.attribute("nonBasicTypeName");
                            string_max_length = case_child.attribute("stringMaxLength");
                            array_dimensions = case_child.attribute("arrayDimensions");
                        }
                    }

                    let m_name = m_name.ok_or(XTypesError::InvalidData)?;
                    let m_type = m_type.ok_or(XTypesError::InvalidData)?;

                    let type_ptr = resolve_member_type(
                        m_type,
                        non_basic_type_name,
                        string_max_length,
                        array_dimensions,
                    )?;
                    let m_name_static = Box::leak(m_name.to_string().into_boxed_str());

                    let member_desc = MemberDescriptor {
                        name: m_name_static,
                        id: member_id,
                        r#type: type_ptr,
                        default_value: None,
                        index: member_id,
                        label,
                        try_construct_kind: TryConstructKind::Discard,
                        is_key: false,
                        is_optional: false,
                        is_must_understand: false,
                        is_shared: false,
                        is_default_label,
                        is_external: false,
                    };

                    builder.add_member(member_desc)?;
                    member_id += 1;
                }
            }
        } else if is_enum {
            for child in target_node.children() {
                if child.is_element() && child.tag_name().name() == "enumerator" {
                    let m_name = child.attribute("name").ok_or(XTypesError::InvalidData)?;
                    let value = child.attribute("value").ok_or(XTypesError::InvalidData)?;
                    let label = value.parse().map_err(|_| XTypesError::InvalidData)?;

                    let type_ptr: DynamicType = Self::get_primitive_type(TypeKind::INT32);
                    let m_name_static = Box::leak(m_name.to_string().into_boxed_str());

                    let member_desc = MemberDescriptor {
                        name: m_name_static,
                        id: member_id,
                        r#type: type_ptr,
                        default_value: None,
                        index: member_id,
                        label: Some(label),
                        try_construct_kind: TryConstructKind::Discard,
                        is_key: false,
                        is_optional: false,
                        is_must_understand: false,
                        is_shared: false,
                        is_default_label: false,
                        is_external: false,
                    };

                    builder.add_member(member_desc)?;
                    member_id += 1;
                }
            }
        } else {
            for child in target_node.children() {
                if child.is_element() && child.tag_name().name() == "member" {
                    let m_name = child.attribute("name").ok_or(XTypesError::InvalidData)?;
                    let m_type = child.attribute("type").ok_or(XTypesError::InvalidData)?;
                    let non_basic_type_name = child.attribute("nonBasicTypeName");
                    let string_max_length = child.attribute("stringMaxLength");
                    let array_dimensions = child.attribute("arrayDimensions");

                    let type_ptr = resolve_member_type(
                        m_type,
                        non_basic_type_name,
                        string_max_length,
                        array_dimensions,
                    )?;
                    let m_name_static = Box::leak(m_name.to_string().into_boxed_str());

                    let member_desc = MemberDescriptor {
                        name: m_name_static,
                        id: member_id,
                        r#type: type_ptr,
                        default_value: None,
                        index: member_id,
                        label: None,
                        try_construct_kind: TryConstructKind::Discard,
                        is_key: child.attribute("key") == Some("true"),
                        is_optional: child.attribute("optional") == Some("true"),
                        is_must_understand: false,
                        is_shared: false,
                        is_default_label: false,
                        is_external: false,
                    };

                    builder.add_member(member_desc)?;
                    member_id += 1;
                }
            }
        }

        Ok(builder)
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
    pub base_type: Option<DynamicType>,
    pub discriminator_type: Option<DynamicType>,
    pub bound: BoundSeq,
    pub element_type: Option<DynamicType>,
    pub key_element_type: Option<DynamicType>,
    pub extensibility_kind: ExtensibilityKind,
    pub is_nested: bool,
}

pub type MemberId = u32;
pub type UnionCaseLabelSeq = Option<i32>;

pub struct MemberDescriptor {
    pub name: ObjectName<'static>,
    pub id: MemberId,
    pub r#type: DynamicType,
    pub default_value: Option<&'static str>,
    pub index: u32,
    pub label: UnionCaseLabelSeq,
    pub try_construct_kind: TryConstructKind,
    pub is_key: bool,
    pub is_optional: bool,
    pub is_must_understand: bool,
    pub is_shared: bool,
    pub is_default_label: bool,
    pub is_external: bool,
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

    pub fn build(self) -> DynamicType {
        DynamicType {
            descriptor: Box::leak(Box::new(self.descriptor)),
            member_list: Vec::leak(self.member_list),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DynamicType {
    pub descriptor: &'static TypeDescriptor,
    pub member_list: &'static [DynamicTypeMember],
}

impl PartialEq for DynamicType {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::addr_eq(self.descriptor, other.descriptor)
            && core::ptr::addr_eq(self.member_list, other.member_list)
    }
}

impl DynamicType {
    pub fn get_descriptor(&self) -> &TypeDescriptor {
        self.descriptor
    }
    pub fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
    }
    pub fn get_kind(&self) -> TypeKind {
        self.descriptor.kind
    }

    pub fn get_member_by_name(&self, name: ObjectName) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .iter()
            .find(|m| m.get_name() == name)
            .ok_or(XTypesError::InvalidName)
    }

    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    pub fn get_member(&self, id: MemberId) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .iter()
            .find(|m| m.get_id() == id)
            .ok_or(XTypesError::InvalidId(id))
    }

    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);

    pub fn get_member_count(&self) -> u32 {
        self.member_list.len() as u32
    }
    pub fn get_member_by_index(&self, index: u32) -> Result<&DynamicTypeMember, XTypesError> {
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
    pub fn create_data(r#type: DynamicType) -> DynamicData {
        DynamicData {
            r#type,
            abstract_data: BTreeMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct DynamicData {
    r#type: DynamicType,
    abstract_data: BTreeMap<MemberId, DataStorage>,
}

impl core::fmt::Debug for DynamicData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DynamicData")
            .field("abstract_data", &self.abstract_data)
            .finish()
    }
}

impl PartialEq for DynamicData {
    fn eq(&self, other: &Self) -> bool {
        self.abstract_data == other.abstract_data
    }
}

impl DynamicData {
    pub fn r#type(&self) -> DynamicType {
        self.r#type
    }

    pub fn get_descriptor(&self, id: MemberId) -> XTypesResult<&MemberDescriptor> {
        if let Ok(x) = self.r#type.get_member(id) {
            Ok(&x.descriptor)
        } else if let Some(b) = &self.r#type.descriptor.base_type {
            b.get_member(id)?.get_descriptor()
        } else {
            Err(XTypesError::InvalidId(id))
        }
    }

    pub fn set_descriptor(&mut self, _id: MemberId, _value: MemberDescriptor) -> XTypesResult<()> {
        todo!()
    }

    pub fn get_member_id_by_name(&self, name: &str) -> Option<MemberId> {
        self.r#type
            .get_member_by_name(name)
            .ok()
            .map(|m| m.get_id())
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

    pub fn clear_nonkey_values(&mut self) -> XTypesResult<()> {
        for index in 0..self.r#type.get_member_count() {
            let member = self.r#type.get_member_by_index(index)?;
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

    pub fn get_float128_value(&self, id: MemberId) -> XTypesResult<&i128> {
        if let DataStorage::Float128(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d)
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float128_value(&mut self, id: MemberId, value: i128) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Float128(value));
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

    pub fn get_float128_values(&self, id: MemberId) -> XTypesResult<&[i128]> {
        if let DataStorage::SequenceFloat128(d) = self
            .abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))?
        {
            Ok(d.as_slice())
        } else {
            Err(XTypesError::InvalidType)
        }
    }

    pub fn set_float128_values(&mut self, id: MemberId, value: Vec<i128>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceFloat128(value));
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

impl TypeSupport for DynamicData {
    const r#TYPE: DynamicType = DynamicType {
        descriptor: &dust_dds::xtypes::dynamic_type::TypeDescriptor {
            kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: None,
            element_type: None,
            key_element_type: None,
            extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };

    fn create_sample(src: &mut DynamicData) -> Self {
        src.clone()
    }

    fn create_dynamic_sample(self, data: &mut DynamicData) {
        *data = self;
    }
}

#[cfg(feature = "xtypes-xml")]
impl DynamicData {
    pub fn from_xml(&mut self, xml: &str) -> XTypesResult<()> {
        let doc = roxmltree::Document::parse(xml).map_err(|_| XTypesError::InvalidData)?;
        let root = doc.root_element();
        self.populate_from_xml_node(root)
    }

    fn populate_from_xml_node(&mut self, node: roxmltree::Node) -> XTypesResult<()> {
        for child in node.children().filter(|c| c.is_element()) {
            let tag_name = child.tag_name().name();

            if tag_name == "discriminator" && self.r#type.get_kind() == TypeKind::UNION {
                continue;
            }

            if let Ok(member) = self.r#type.get_member_by_name(tag_name) {
                let member_id = member.get_id();
                let member_type = member.get_descriptor()?.r#type;

                let data = Self::parse_xml_node_to_data(child, member_type)?;
                self.set_value(member_id, data);
            }
        }
        Ok(())
    }

    fn parse_xml_node_to_data(
        node: roxmltree::Node,
        r#type: DynamicType,
    ) -> XTypesResult<DataStorage> {
        let kind = r#type.get_kind();
        let text = node.text().unwrap_or("").trim();

        let parse_int = |s: &str| -> Result<i64, ()> {
            if let Some(hex) = s.strip_prefix("0x") {
                i64::from_str_radix(hex, 16).map_err(|_| ())
            } else {
                s.parse::<i64>().map_err(|_| ())
            }
        };

        let parse_uint = |s: &str| -> Result<u64, ()> {
            if let Some(hex) = s.strip_prefix("0x") {
                u64::from_str_radix(hex, 16).map_err(|_| ())
            } else {
                s.parse::<u64>().map_err(|_| ())
            }
        };

        match kind {
            TypeKind::BOOLEAN => {
                let val = text == "true" || text == "1";
                Ok(DataStorage::Boolean(val))
            }
            TypeKind::BYTE | TypeKind::UINT8 => {
                let val = parse_uint(text).map_err(|_| XTypesError::InvalidData)? as u8;
                Ok(DataStorage::UInt8(val))
            }
            TypeKind::INT8 => {
                let val = parse_int(text).map_err(|_| XTypesError::InvalidData)? as i8;
                Ok(DataStorage::Int8(val))
            }
            TypeKind::UINT16 => {
                let val = parse_uint(text).map_err(|_| XTypesError::InvalidData)? as u16;
                Ok(DataStorage::UInt16(val))
            }
            TypeKind::INT16 => {
                let val = parse_int(text).map_err(|_| XTypesError::InvalidData)? as i16;
                Ok(DataStorage::Int16(val))
            }
            TypeKind::UINT32 => {
                let val = parse_uint(text).map_err(|_| XTypesError::InvalidData)? as u32;
                Ok(DataStorage::UInt32(val))
            }
            TypeKind::INT32 => {
                let val = parse_int(text).map_err(|_| XTypesError::InvalidData)? as i32;
                Ok(DataStorage::Int32(val))
            }
            TypeKind::UINT64 => {
                let val = parse_uint(text).map_err(|_| XTypesError::InvalidData)?;
                Ok(DataStorage::UInt64(val))
            }
            TypeKind::INT64 => {
                let val = parse_int(text).map_err(|_| XTypesError::InvalidData)?;
                Ok(DataStorage::Int64(val))
            }
            TypeKind::FLOAT32 => {
                let val = text.parse::<f32>().map_err(|_| XTypesError::InvalidData)?;
                Ok(DataStorage::Float32(val))
            }
            TypeKind::FLOAT64 => {
                let val = text.parse::<f64>().map_err(|_| XTypesError::InvalidData)?;
                Ok(DataStorage::Float64(val))
            }
            TypeKind::FLOAT128 => {
                let val = text
                    .parse::<f32>()
                    .map_err(|_| XTypesError::InvalidData)?
                    .to_be_bytes();
                Ok(DataStorage::Float128(i128::from_be_bytes([
                    val[3], val[2], val[1], val[0], 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ])))
            }
            TypeKind::CHAR8 => {
                let val = parse_uint(text)
                    .ok()
                    .map(|v| v as u8 as char)
                    .unwrap_or_else(|| text.chars().next().unwrap_or('\0'));
                Ok(DataStorage::Char8(val))
            }
            TypeKind::STRING8 => {
                let val = node.text().unwrap_or("");
                Ok(DataStorage::String(String::from(val)))
            }
            TypeKind::STRUCTURE | TypeKind::UNION => {
                let mut inner_data = DynamicDataFactory::create_data(r#type);
                inner_data.populate_from_xml_node(node)?;
                Ok(DataStorage::ComplexValue(inner_data))
            }
            TypeKind::SEQUENCE | TypeKind::ARRAY => {
                let element_type = r#type
                    .get_descriptor()
                    .element_type
                    .ok_or(XTypesError::InvalidData)?;
                let element_kind = element_type.get_kind();

                macro_rules! parse_seq {
                    ($parse_fn:ident, $storage_variant:ident, $cast_type:ty) => {{
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let item_text = item.text().unwrap_or("").trim();
                            let val = $parse_fn(item_text).map_err(|_| XTypesError::InvalidData)?
                                as $cast_type;
                            vec.push(val);
                        }
                        Ok(DataStorage::$storage_variant(vec))
                    }};
                }

                match element_kind {
                    TypeKind::INT32 => parse_seq!(parse_int, SequenceInt32, i32),
                    TypeKind::UINT32 => parse_seq!(parse_uint, SequenceUInt32, u32),
                    TypeKind::INT8 => parse_seq!(parse_int, SequenceInt8, i8),
                    TypeKind::UINT8 | TypeKind::BYTE => {
                        parse_seq!(parse_uint, SequenceUInt8, u8)
                    }
                    TypeKind::INT16 => parse_seq!(parse_int, SequenceInt16, i16),
                    TypeKind::UINT16 => parse_seq!(parse_uint, SequenceUInt16, u16),
                    TypeKind::INT64 => parse_seq!(parse_int, SequenceInt64, i64),
                    TypeKind::UINT64 => parse_seq!(parse_uint, SequenceUInt64, u64),
                    TypeKind::FLOAT32 => {
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let item_text = item.text().unwrap_or("").trim();
                            vec.push(
                                item_text
                                    .parse::<f32>()
                                    .map_err(|_| XTypesError::InvalidData)?,
                            );
                        }
                        Ok(DataStorage::SequenceFloat32(vec))
                    }
                    TypeKind::FLOAT64 => {
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let item_text = item.text().unwrap_or("").trim();
                            vec.push(
                                item_text
                                    .parse::<f64>()
                                    .map_err(|_| XTypesError::InvalidData)?,
                            );
                        }
                        Ok(DataStorage::SequenceFloat64(vec))
                    }
                    TypeKind::BOOLEAN => {
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let item_text = item.text().unwrap_or("").trim();
                            vec.push(item_text == "true" || item_text == "1");
                        }
                        Ok(DataStorage::SequenceBoolean(vec))
                    }
                    TypeKind::CHAR8 => {
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let item_text = item.text().unwrap_or("");
                            let val = parse_uint(item_text)
                                .ok()
                                .map(|v| v as u8 as char)
                                .unwrap_or_else(|| item_text.chars().next().unwrap_or('\0'));
                            vec.push(val);
                        }
                        Ok(DataStorage::SequenceChar8(vec))
                    }
                    TypeKind::STRING8 => {
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let item_text = item.text().unwrap_or("");
                            vec.push(String::from(item_text));
                        }
                        Ok(DataStorage::SequenceString(vec))
                    }
                    TypeKind::STRUCTURE | TypeKind::UNION => {
                        let mut vec = Vec::new();
                        for item in node.children().filter(|c| c.is_element()) {
                            let mut inner_data = DynamicDataFactory::create_data(element_type);
                            inner_data.populate_from_xml_node(item)?;
                            vec.push(inner_data);
                        }
                        Ok(DataStorage::SequenceComplexValue(vec))
                    }
                    _ => Err(XTypesError::InvalidData),
                }
            }
            _ => Err(XTypesError::InvalidData),
        }
    }
}
