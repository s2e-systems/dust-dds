use super::error::XTypesError;
use crate::xtypes::{
    data_storage::{DataStorage, DataStorageMapping},
    error::XTypesResult,
    type_object::TypeObject,
    type_support::{Type, TypeSupport},
};
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec, vec::Vec};

/// Represents a sequence bound.
pub type BoundSeq<'a> = &'a [u32];
/// Represents a sequence of include paths.
pub type IncludePathSeq = Vec<String>;
/// Represents the name of an object.
pub type ObjectName<'a> = &'a str;

// ---------- TypeKinds (begin) -------------------
/// Represents the kind of a dynamic type (e.g., primitive, constructed, or collection type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TypeKind {
    /// No type kind.
    NONE = 0x00,
    /// Boolean type kind.
    BOOLEAN = 0x01,
    /// Byte type kind.
    BYTE = 0x02,
    /// 16-bit signed integer type kind.
    INT16 = 0x03,
    /// 32-bit signed integer type kind.
    INT32 = 0x04,
    /// 64-bit signed integer type kind.
    INT64 = 0x05,
    /// 16-bit unsigned integer type kind.
    UINT16 = 0x06,
    /// 32-bit unsigned integer type kind.
    UINT32 = 0x07,
    /// 64-bit unsigned integer type kind.
    UINT64 = 0x08,
    /// 32-bit floating point type kind.
    FLOAT32 = 0x09,
    /// 64-bit floating point type kind.
    FLOAT64 = 0x0A,
    /// 128-bit floating point type kind.
    FLOAT128 = 0x0B,
    /// 8-bit signed integer type kind.
    INT8 = 0x0C,
    /// 8-bit unsigned integer type kind.
    UINT8 = 0x0D,
    /// 8-bit character type kind.
    CHAR8 = 0x10,
    /// 16-bit character type kind.
    CHAR16 = 0x11,
    /// 8-bit string type kind.
    STRING8 = 0x20,
    /// 16-bit string type kind.
    STRING16 = 0x21,
    /// Alias type kind.
    ALIAS = 0x30,
    /// Enumeration type kind.
    ENUM = 0x40,
    /// Bitmask type kind.
    BITMASK = 0x41,
    /// Annotation type kind.
    ANNOTATION = 0x50,
    /// Structure type kind.
    STRUCTURE = 0x51,
    /// Union type kind.
    UNION = 0x52,
    /// Bitset type kind.
    BITSET = 0x53,
    /// Sequence type kind.
    SEQUENCE = 0x60,
    /// Array type kind.
    ARRAY = 0x61,
    /// Map type kind.
    MAP = 0x62,
}

// ---------- TypeKinds (end) -------------------

/// A factory class used to create [`DynamicType`] and [`DynamicTypeBuilder`] instances.
pub struct DynamicTypeBuilderFactory;

impl DynamicTypeBuilderFactory {
    /// Returns a [`DynamicType`] representing the specified primitive type kind.
    pub fn get_primitive_type(kind: TypeKind) -> DynamicType<'static> {
        DynamicType {
            descriptor: Box::leak(Box::new(TypeDescriptor {
                kind,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: &[],
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            })),
            member_list: &[],
        }
    }

    /// Creates a [`DynamicTypeBuilder`] with the given type descriptor.
    pub fn create_type(descriptor: TypeDescriptor) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor,
            member_list: Vec::new(),
        }
    }

    /// Creates a [`DynamicTypeBuilder`] as a copy of an existing type.
    pub fn create_type_copy(r#_type: DynamicType) -> DynamicTypeBuilder {
        todo!()
    }

    /// Creates a [`DynamicTypeBuilder`] from a [`TypeObject`].
    pub fn create_type_w_type_object(_type_object: TypeObject) -> DynamicTypeBuilder {
        todo!()
    }

    /// Creates a [`DynamicTypeBuilder`] for a string type with the specified bound.
    pub fn create_string_type(bound: u32) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TypeKind::STRING8,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: vec![bound].leak(),
                element_type: None,
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            member_list: Vec::new(),
        }
    }

    /// Creates a [`DynamicTypeBuilder`] for a wide string type with the specified bound.
    pub fn create_wstring_type(_bound: u32) -> DynamicTypeBuilder {
        unimplemented!("wstring not supported in Rust")
    }

    /// Creates a [`DynamicTypeBuilder`] for a sequence type with the specified element type and bound.
    pub fn create_sequence_type(
        element_type: DynamicType<'static>,
        bound: u32,
    ) -> DynamicTypeBuilder {
        DynamicTypeBuilder {
            descriptor: TypeDescriptor {
                kind: TypeKind::SEQUENCE,
                name: "",
                base_type: None,
                discriminator_type: None,
                bound: vec![bound].leak(),
                element_type: Some(element_type),
                key_element_type: None,
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            },
            member_list: Vec::new(),
        }
    }

    /// Creates a [`DynamicTypeBuilder`] for an array type with the specified element type and dimensions/bound.
    pub fn create_array_type(
        element_type: DynamicType<'static>,
        bound: BoundSeq<'static>,
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

    /// Creates a [`DynamicTypeBuilder`] for a map type with the specified key, element type, and bound.
    pub fn create_map_type(
        _key_element_type: DynamicType,
        _element_type: DynamicType,
        _bound: u32,
    ) -> DynamicTypeBuilder {
        todo!()
    }

    /// Creates a [`DynamicTypeBuilder`] for a bitmask type with the specified bound.
    pub fn create_bitmask_type(_bound: u32) -> DynamicTypeBuilder {
        todo!()
    }

    /// Creates a [`DynamicTypeBuilder`] for a type defined at the specified URI.
    pub fn create_type_w_uri(
        _document_url: String,
        _type_name: String,
        _include_paths: Vec<String>,
    ) -> DynamicTypeBuilder {
        todo!()
    }

    #[cfg(feature = "xtypes-xml")]
    /// Creates a [`DynamicTypeBuilder`] for a type defined by the input XML.
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
                let builder = Self::create_array_type(type_ptr, dims.leak());
                type_ptr = builder.build();
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
            bound: &[],
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
                    let mut label = Vec::new();
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
                                label.push(
                                    i32::from_str_radix(hex, 16)
                                        .map_err(|_| XTypesError::InvalidData)?,
                                );
                            } else {
                                label.push(
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
                    let label = Vec::leak(label);
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
                    let label =
                        Vec::leak(vec![value.parse().map_err(|_| XTypesError::InvalidData)?]);

                    let type_ptr: DynamicType = Self::get_primitive_type(TypeKind::INT32);
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
                        label: &[],
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

/// Represents parameter name-value pairs.
pub type Parameters = BTreeMap<ObjectName<'static>, ObjectName<'static>>;

/// Defines how a type can be extended or modified in future versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensibilityKind {
    /// Cannot be extended or modified.
    Final,
    /// Members can be appended to the end of the type in future versions.
    Appendable,
    /// Members can be added, removed, or reordered in future versions.
    Mutable,
}

/// Defines the behavior when constructing an object of a type that fails some validation or constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryConstructKind {
    /// Fall back to the default value.
    UseDefault,
    /// Discard the entire object or element.
    Discard,
    /// Trim the elements to fit the constraints.
    Trim,
}

/// Describes the properties and characteristics of a [`DynamicType`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDescriptor {
    /// The kind of the type.
    pub kind: TypeKind,
    /// The name of the type.
    pub name: ObjectName<'static>,
    /// The base type if this type inherits from another type.
    pub base_type: Option<DynamicType<'static>>,
    /// The discriminator type if this type is a union.
    pub discriminator_type: Option<DynamicType<'static>>,
    /// The bound(s) of the type if it is a collection or string.
    pub bound: BoundSeq<'static>,
    /// The element type if this type is a collection.
    pub element_type: Option<DynamicType<'static>>,
    /// The key element type if this type is a map.
    pub key_element_type: Option<DynamicType<'static>>,
    /// The extensibility kind of the type.
    pub extensibility_kind: ExtensibilityKind,
    /// Indicates whether this is a nested type.
    pub is_nested: bool,
}

/// Represents the unique identifier of a member.
pub type MemberId = u32;
/// Represents case labels for a union member.
pub type UnionCaseLabelSeq = &'static [i32];

/// Describes a member of a constructed type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberDescriptor {
    /// The name of the member.
    pub name: ObjectName<'static>,
    /// The unique identifier of the member.
    pub id: MemberId,
    /// The dynamic type of the member.
    pub r#type: DynamicType<'static>,
    /// The optional default value of the member.
    pub default_value: Option<&'static str>,
    /// The index of the member within the parent type.
    pub index: u32,
    /// The union case labels if this member belongs to a union.
    pub label: UnionCaseLabelSeq,
    /// The construct fail action of the member.
    pub try_construct_kind: TryConstructKind,
    /// Indicates if the member is part of the type's key.
    pub is_key: bool,
    /// Indicates if the member is optional.
    pub is_optional: bool,
    /// Indicates if a receiver must understand this member to process the data.
    pub is_must_understand: bool,
    /// Indicates if the member is shared.
    pub is_shared: bool,
    /// Indicates if this is the default case for a union.
    pub is_default_label: bool,
    /// Indicates if the member is external (stored by reference).
    pub is_external: bool,
}

/// Represents a member of a [`DynamicType`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicTypeMember {
    /// The descriptor describing this member.
    pub descriptor: MemberDescriptor,
}

impl DynamicTypeMember {
    /// Returns a reference to the member's descriptor.
    pub fn get_descriptor(&self) -> XTypesResult<&MemberDescriptor> {
        Ok(&self.descriptor)
    }
    // unsigned long get_annotation_count();
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);

    /// Returns the unique member ID.
    pub fn get_id(&self) -> MemberId {
        self.descriptor.id
    }
    /// Returns the name of the member.
    pub fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
    }
}

/// A builder class used to construct a [`DynamicType`].
pub struct DynamicTypeBuilder {
    descriptor: TypeDescriptor,
    member_list: Vec<DynamicTypeMember>,
}

impl DynamicTypeBuilder {
    /// Returns a reference to the type descriptor being built.
    pub fn get_descriptor(&self) -> XTypesResult<&TypeDescriptor> {
        Ok(&self.descriptor)
    }

    /// Returns the name of the type.
    pub fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
    }

    /// Returns the kind of the type.
    pub fn get_kind(&self) -> TypeKind {
        self.descriptor.kind
    }

    /// Returns a mutable reference to a member by its name.
    pub fn get_member_by_name(
        &mut self,
        name: &ObjectName,
    ) -> XTypesResult<&mut DynamicTypeMember> {
        self.member_list
            .iter_mut()
            .find(|m| &m.descriptor.name == name)
            .ok_or(XTypesError::InvalidData)
    }

    /// Returns all members indexed by their names.
    pub fn get_all_members_by_name(
        &self,
    ) -> Result<Vec<(ObjectName<'static>, DynamicTypeMember)>, XTypesError> {
        todo!()
    }

    /// Returns a member by its ID.
    pub fn get_member(&self, _id: MemberId) -> Result<DynamicTypeMember, XTypesError> {
        todo!()
    }

    /// Returns all members indexed by their ID.
    pub fn get_all_members(
        &mut self,
    ) -> Result<Vec<(MemberId, &mut DynamicTypeMember)>, XTypesError> {
        Ok(self
            .member_list
            .iter_mut()
            .map(|m| (m.descriptor.id, m))
            .collect())
    }

    /// Returns the number of annotations on this type.
    pub fn get_annotation_count(&self) -> u32 {
        todo!()
    }

    /// Returns the annotation at the specified index.
    pub fn get_annotation(&self, _idx: u32) -> XTypesResult<()> {
        todo!()
    }

    /// Adds a member to the type.
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

    /// Applies an annotation descriptor to this type.
    pub fn apply_annotation(&mut self) -> XTypesResult<()> {
        todo!()
    }

    /// Builds and returns the constructed [`DynamicType`].
    pub fn build(self) -> DynamicType<'static> {
        DynamicType {
            descriptor: Box::leak(Box::new(self.descriptor)),
            member_list: Vec::leak(self.member_list),
        }
    }
}

/// Represents a data type's schema at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DynamicType<'a> {
    /// The type descriptor.
    pub descriptor: &'a TypeDescriptor,
    /// The list of members belonging to this type.
    pub member_list: &'a [DynamicTypeMember],
}

impl<'a> DynamicType<'a> {
    /// Returns the type descriptor.
    pub fn get_descriptor(&self) -> &TypeDescriptor {
        self.descriptor
    }
    /// Returns the name of the type.
    pub fn get_name(&self) -> ObjectName<'static> {
        self.descriptor.name
    }
    /// Returns the kind of the type.
    pub fn get_kind(&self) -> TypeKind {
        self.descriptor.kind
    }

    /// Retrieves a member by name.
    pub fn get_member_by_name(&self, name: ObjectName) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .iter()
            .find(|m| m.get_name() == name)
            .ok_or(XTypesError::InvalidName)
    }

    // DDS::ReturnCode_t get_all_members_by_name(inout DynamicTypeMembersByName member);
    /// Retrieves a member by ID.
    pub fn get_member(&self, id: MemberId) -> Result<&DynamicTypeMember, XTypesError> {
        self.member_list
            .iter()
            .find(|m| m.get_id() == id)
            .ok_or(XTypesError::InvalidId(id))
    }

    // DDS::ReturnCode_t get_all_members(inout DynamicTypeMembersById member);

    /// Returns the total number of members in the type.
    pub fn get_member_count(&self) -> u32 {
        self.member_list.len() as u32
    }
    /// Retrieves a member by index.
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

/// A factory class used to instantiate [`DynamicData`] samples.
pub struct DynamicDataFactory;

impl DynamicDataFactory {
    /// Creates a [`DynamicData`] sample of the specified [`DynamicType`].
    pub fn create_data<'a>(r#type: DynamicType<'a>) -> DynamicData<'a> {
        DynamicData {
            r#type,
            abstract_data: BTreeMap::new(),
        }
    }
}

/// Represents a data sample conforming to a [`DynamicType`] schema.
#[derive(Clone)]
pub struct DynamicData<'a> {
    r#type: DynamicType<'a>,
    abstract_data: BTreeMap<MemberId, DataStorage>,
}

impl<'a> core::fmt::Debug for DynamicData<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DynamicData")
            .field("abstract_data", &self.abstract_data)
            .finish()
    }
}

impl<'a> PartialEq for DynamicData<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.abstract_data == other.abstract_data
    }
}

impl<'a> DynamicData<'a> {
    /// Returns the [`DynamicType`] of this data sample.
    pub const fn r#type(&self) -> DynamicType<'a> {
        self.r#type
    }

    /// Returns a reference to the descriptor of the specified member.
    pub fn get_descriptor(&self, id: MemberId) -> XTypesResult<&MemberDescriptor> {
        if let Ok(x) = self.r#type.get_member(id) {
            Ok(&x.descriptor)
        } else if let Some(b) = &self.r#type.descriptor.base_type {
            b.get_member(id)?.get_descriptor()
        } else {
            Err(XTypesError::InvalidId(id))
        }
    }

    /// Sets the descriptor of the specified member.
    pub fn set_descriptor(&mut self, _id: MemberId, _value: MemberDescriptor) -> XTypesResult<()> {
        todo!()
    }

    /// Retrieves the member ID corresponding to the given member name.
    pub fn get_member_id_by_name(&self, name: &str) -> Option<MemberId> {
        self.r#type
            .get_member_by_name(name)
            .ok()
            .map(|m| m.get_id())
    }

    /// Retrieves the member ID at the specified index.
    pub fn get_member_id_at_index(&self, index: u32) -> XTypesResult<MemberId> {
        self.abstract_data
            .keys()
            .nth(index as usize)
            .cloned()
            .ok_or(XTypesError::InvalidIndex(index))
    }

    /// Returns the number of items/members in this data sample.
    pub fn get_item_count(&self) -> u32 {
        self.abstract_data.len() as u32
    }

    /// Clears all member values from this data sample.
    pub fn clear_all_values(&mut self) -> XTypesResult<()> {
        self.abstract_data.clear();
        Ok(())
    }

    /// Clears all non-key member values from this data sample.
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

    /// Clears the value of the specified member.
    pub fn clear_value(&mut self, id: MemberId) -> XTypesResult<()> {
        self.abstract_data
            .remove(&id)
            .ok_or(XTypesError::InvalidId(id))?;
        Ok(())
    }

    /// Gets the `i32` value for the specified member.
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

    /// Sets the `i32` value for the specified member.
    pub fn set_int32_value(&mut self, id: MemberId, value: i32) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int32(value));
        Ok(())
    }

    /// Gets the `u32` value for the specified member.
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

    /// Sets the `u32` value for the specified member.
    pub fn set_uint32_value(&mut self, id: MemberId, value: u32) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt32(value));
        Ok(())
    }

    /// Gets the `i8` value for the specified member.
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

    /// Sets the `i8` value for the specified member.
    pub fn set_int8_value(&mut self, id: MemberId, value: i8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int8(value));
        Ok(())
    }

    /// Gets the `u8` value for the specified member.
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

    /// Sets the `u8` value for the specified member.
    pub fn set_uint8_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt8(value));
        Ok(())
    }

    /// Gets the `i16` value for the specified member.
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

    /// Sets the `i16` value for the specified member.
    pub fn set_int16_value(&mut self, id: MemberId, value: i16) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int16(value));
        Ok(())
    }

    /// Gets the `u16` value for the specified member.
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

    /// Sets the `u16` value for the specified member.
    pub fn set_uint16_value(&mut self, id: MemberId, value: u16) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt16(value));
        Ok(())
    }

    /// Gets the `i64` value for the specified member.
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

    /// Sets the `i64` value for the specified member.
    pub fn set_int64_value(&mut self, id: MemberId, value: i64) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Int64(value));
        Ok(())
    }

    /// Gets the `u64` value for the specified member.
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

    /// Sets the `u64` value for the specified member.
    pub fn set_uint64_value(&mut self, id: MemberId, value: u64) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt64(value));
        Ok(())
    }

    /// Gets the `f32` value for the specified member.
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

    /// Sets the `f32` value for the specified member.
    pub fn set_float32_value(&mut self, id: MemberId, value: f32) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Float32(value));
        Ok(())
    }

    /// Gets the `f64` value for the specified member.
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

    /// Sets the `f64` value for the specified member.
    pub fn set_float64_value(&mut self, id: MemberId, value: f64) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Float64(value));
        Ok(())
    }

    /// Gets the `i128` (representing `float128`) value for the specified member.
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

    /// Sets the `i128` (representing `float128`) value for the specified member.
    pub fn set_float128_value(&mut self, id: MemberId, value: i128) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Float128(value));
        Ok(())
    }

    /// Gets the `char` (representing `char8`) value for the specified member.
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

    /// Sets the `char` (representing `char8`) value for the specified member.
    pub fn set_char8_value(&mut self, id: MemberId, value: char) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::Char8(value));
        Ok(())
    }

    /// Gets the byte (8-bit unsigned integer) value for the specified member.
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

    /// Sets the byte (8-bit unsigned integer) value for the specified member.
    pub fn set_byte_value(&mut self, id: MemberId, value: u8) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::UInt8(value));
        Ok(())
    }

    /// Gets the `bool` value for the specified member.
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

    /// Sets the `bool` value for the specified member.
    pub fn set_boolean_value(&mut self, id: MemberId, value: bool) -> XTypesResult<()> {
        self.abstract_data.insert(id, value.into_storage());
        Ok(())
    }

    /// Gets the `String` value for the specified member.
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

    /// Sets the `String` value for the specified member.
    pub fn set_string_value(&mut self, id: MemberId, value: String) -> XTypesResult<()> {
        self.abstract_data.insert(id, DataStorage::String(value));
        Ok(())
    }

    /// Gets the complex (nested `DynamicData`) value for the specified member.
    pub fn get_complex_value(&self, id: MemberId) -> XTypesResult<&DynamicData<'static>> {
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

    /// Gets the raw data kind/storage for the specified member.
    pub fn get_data_kind(&self, id: MemberId) -> XTypesResult<&DataStorage> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))
    }

    /// Sets the complex (nested `DynamicData`) value for the specified member.
    pub fn set_complex_value(
        &mut self,
        id: MemberId,
        value: DynamicData<'static>,
    ) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::ComplexValue(value));
        Ok(())
    }

    /// Gets a slice of `i32` values for the specified sequence/array member.
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

    /// Sets a sequence of `i32` values for the specified member.
    pub fn set_int32_values(&mut self, id: MemberId, value: Vec<i32>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceInt32(value));
        Ok(())
    }

    /// Gets a slice of `u32` values for the specified sequence/array member.
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

    /// Sets a sequence of `u32` values for the specified member.
    pub fn set_uint32_values(&mut self, id: MemberId, value: Vec<u32>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceUInt32(value));
        Ok(())
    }

    /// Gets a slice of `i16` values for the specified sequence/array member.
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

    /// Sets a sequence of `i16` values for the specified member.
    pub fn set_int16_values(&mut self, id: MemberId, value: Vec<i16>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceInt16(value));
        Ok(())
    }

    /// Gets a slice of `u16` values for the specified sequence/array member.
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

    /// Sets a sequence of `u16` values for the specified member.
    pub fn set_uint16_values(&mut self, id: MemberId, value: Vec<u16>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceUInt16(value));
        Ok(())
    }

    /// Gets a slice of `i64` values for the specified sequence/array member.
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

    /// Sets a sequence of `i64` values for the specified member.
    pub fn set_int64_values(&mut self, id: MemberId, value: Vec<i64>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceInt64(value));
        Ok(())
    }

    /// Gets a slice of `u64` values for the specified sequence/array member.
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

    /// Sets a sequence of `u64` values for the specified member.
    pub fn set_uint64_values(&mut self, id: MemberId, value: Vec<u64>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceUInt64(value));
        Ok(())
    }

    /// Gets a slice of `f32` values for the specified sequence/array member.
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

    /// Sets a sequence of `f32` values for the specified member.
    pub fn set_float32_values(&mut self, id: MemberId, value: Vec<f32>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceFloat32(value));
        Ok(())
    }

    /// Gets a slice of `f64` values for the specified sequence/array member.
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

    /// Sets a sequence of `f64` values for the specified member.
    pub fn set_float64_values(&mut self, id: MemberId, value: Vec<f64>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceFloat64(value));
        Ok(())
    }

    /// Gets a slice of `i128` (representing `float128`) values for the specified sequence/array member.
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

    /// Sets a sequence of `i128` (representing `float128`) values for the specified member.
    pub fn set_float128_values(&mut self, id: MemberId, value: Vec<i128>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceFloat128(value));
        Ok(())
    }

    /// Gets a slice of `char` (representing `char8`) values for the specified sequence/array member.
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

    /// Sets a sequence of `char` (representing `char8`) values for the specified member.
    pub fn set_char8_values(&mut self, id: MemberId, value: Vec<char>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceChar8(value));
        Ok(())
    }

    /// Gets a slice of byte (8-bit unsigned integer) values for the specified sequence/array member.
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

    /// Sets a sequence of byte (8-bit unsigned integer) values for the specified member.
    pub fn set_byte_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceUInt8(value));
        Ok(())
    }

    /// Gets a slice of `bool` values for the specified sequence/array member.
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

    /// Sets a sequence of `bool` values for the specified member.
    pub fn set_boolean_values(&mut self, id: MemberId, value: Vec<bool>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceBoolean(value));
        Ok(())
    }

    /// Gets a slice of `String` values for the specified sequence/array member.
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

    /// Sets a sequence of `String` values for the specified member.
    pub fn set_string_values(&mut self, id: MemberId, value: Vec<String>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceString(value));
        Ok(())
    }

    // Custom functions
    /// Gets a slice of `u8` values for the specified sequence/array member.
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

    /// Gets a slice of `i8` values for the specified sequence/array member.
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

    /// Sets a sequence of `u8` values for the specified member.
    pub fn set_uint8_values(&mut self, id: MemberId, value: Vec<u8>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceUInt8(value));
        Ok(())
    }

    /// Sets a sequence of `i8` values for the specified member.
    pub fn set_int8_values(&mut self, id: MemberId, value: Vec<i8>) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceInt8(value));
        Ok(())
    }

    /// Gets a slice of complex (nested `DynamicData`) values for the specified sequence/array member.
    pub fn get_complex_values(&self, id: MemberId) -> XTypesResult<&[DynamicData<'static>]> {
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

    /// Sets a sequence of complex (nested `DynamicData`) values for the specified member.
    pub fn set_complex_values(
        &mut self,
        id: MemberId,
        value: Vec<DynamicData<'static>>,
    ) -> XTypesResult<()> {
        self.abstract_data
            .insert(id, DataStorage::SequenceComplexValue(value));
        Ok(())
    }

    /// Sets the value of the specified member to the given raw data storage.
    pub fn set_value(&mut self, id: MemberId, value: DataStorage) {
        self.abstract_data.insert(id, value);
    }

    /// Gets the raw data storage for the specified member.
    pub fn get_value(&self, id: MemberId) -> XTypesResult<&DataStorage> {
        self.abstract_data
            .get(&id)
            .ok_or(XTypesError::InvalidId(id))
    }

    /// Removes and returns the raw data storage for the specified member.
    pub fn remove_value(&mut self, id: MemberId) -> XTypesResult<DataStorage> {
        self.abstract_data
            .remove(&id)
            .ok_or(XTypesError::InvalidId(id))
    }
}

impl Type for DynamicData<'static> {
    const TYPE: DynamicType<'static> = DynamicType {
        descriptor: &dust_dds::xtypes::dynamic_type::TypeDescriptor {
            kind: dust_dds::xtypes::dynamic_type::TypeKind::STRUCTURE,
            name: "",
            base_type: None,
            discriminator_type: None,
            bound: &[],
            element_type: None,
            key_element_type: None,
            extensibility_kind: dust_dds::xtypes::dynamic_type::ExtensibilityKind::Final,
            is_nested: false,
        },
        member_list: &[],
    };
}
impl TypeSupport for DynamicData<'static> {
    fn create_sample(src: &mut DynamicData<'static>) -> Self {
        src.clone()
    }

    fn create_dynamic_sample(self) -> DynamicData<'static> {
        self
    }
}

#[cfg(feature = "xtypes-xml")]
impl<'a> DynamicData<'a> {
    /// Deserializes data from an XML string into this `DynamicData` instance.
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
        r#type: DynamicType<'static>,
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
