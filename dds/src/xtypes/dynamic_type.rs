use super::error::XTypesError;
use crate::xtypes::{
    data_storage::{DataStorage, DataStorageMapping},
    error::XTypesResult,
    type_object::{
        CommonStructMember, CompleteMemberDetail, CompleteStructHeader, CompleteStructMember,
        CompleteStructType, CompleteTypeDetail, CompleteTypeObject, PlainCollectionHeader,
        PlainSequenceLElemDefn, StringLTypeDefn, StructMemberFlag, StructTypeFlag, TypeIdentifier,
        TypeObject, CollectionElementFlag, EK_COMPLETE, INVALID_LBOUND,
        TypeInformation, TypeIdentifierWithDependencies, TypeIdentifierWithSize,
    },
};
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

    /// Creates a DynamicType from a TypeObject.
    ///
    /// This is used for TypeLookup service where types are discovered at runtime
    /// from remote participants and need to be converted to DynamicType for use
    /// with DynamicDataReader.
    pub fn create_type_w_type_object(type_object: TypeObject) -> XTypesResult<DynamicType> {
        match type_object {
            TypeObject::EkComplete { complete } => {
                Self::complete_type_object_to_dynamic_type(complete)
            }
            TypeObject::EkMinimal { .. } => {
                // MinimalTypeObject doesn't contain member names, only name hashes,
                // so it cannot be fully converted to DynamicType without additional info
                Err(XTypesError::IllegalOperation)
            }
        }
    }

    /// Converts a CompleteTypeObject to a DynamicType.
    fn complete_type_object_to_dynamic_type(
        complete: CompleteTypeObject,
    ) -> XTypesResult<DynamicType> {
        match complete {
            CompleteTypeObject::TkStructure { struct_type } => {
                Self::complete_struct_to_dynamic_type(struct_type)
            }
            CompleteTypeObject::TkAlias { .. } => {
                // TODO: Implement alias type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkEnum { .. } => {
                // TODO: Implement enum type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkUnion { .. } => {
                // TODO: Implement union type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkSequence { .. } => {
                // TODO: Implement sequence type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkArray { .. } => {
                // TODO: Implement array type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkMap { .. } => {
                // TODO: Implement map type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkBitmask { .. } => {
                // TODO: Implement bitmask type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkBitset { .. } => {
                // TODO: Implement bitset type conversion
                Err(XTypesError::IllegalOperation)
            }
            CompleteTypeObject::TkAnnotation { .. } => {
                // Annotation types are not typically used for data
                Err(XTypesError::IllegalOperation)
            }
        }
    }

    /// Converts a CompleteStructType to a DynamicType.
    fn complete_struct_to_dynamic_type(struct_type: CompleteStructType) -> XTypesResult<DynamicType> {
        // Determine extensibility from struct flags
        let extensibility_kind = if struct_type.struct_flags.is_final {
            ExtensibilityKind::Final
        } else if struct_type.struct_flags.is_mutable {
            ExtensibilityKind::Mutable
        } else if struct_type.struct_flags.is_appendable {
            ExtensibilityKind::Appendable
        } else {
            ExtensibilityKind::Final // Default
        };

        let mut builder = Self::create_type(TypeDescriptor {
            kind: TypeKind::STRUCTURE,
            name: struct_type.header.detail.type_name,
            base_type: None, // TODO: Handle base_type from struct_type.header.base_type
            discriminator_type: None,
            bound: Vec::new(),
            element_type: None,
            key_element_type: None,
            extensibility_kind,
            is_nested: struct_type.struct_flags.is_nested,
        });

        // Add members
        for (index, member) in struct_type.member_seq.into_iter().enumerate() {
            let member_type = Self::type_identifier_to_dynamic_type(&member.common.member_type_id)?;

            // Extract flags from member_flags
            let try_construct_kind = member.common.member_flags.try_construct;
            let is_key = member.common.member_flags.is_key;
            let is_optional = member.common.member_flags.is_optional;
            let is_must_understand = member.common.member_flags.is_must_undestand;

            builder.add_member(MemberDescriptor {
                name: member.detail.name,
                id: member.common.member_id,
                r#type: member_type,
                default_value: None,
                index: index as u32,
                label: Vec::new(),
                try_construct_kind,
                is_key,
                is_optional,
                is_must_understand,
                is_shared: false,
                is_default_label: false,
            })?;
        }

        Ok(builder.build())
    }

    /// Converts a TypeIdentifier to a DynamicType.
    pub fn type_identifier_to_dynamic_type(type_id: &TypeIdentifier) -> XTypesResult<DynamicType> {
        match type_id {
            // Primitive types
            TypeIdentifier::TkNone => Ok(Self::get_primitive_type(TypeKind::NONE)),
            TypeIdentifier::TkBoolean => Ok(Self::get_primitive_type(TypeKind::BOOLEAN)),
            TypeIdentifier::TkByteType => Ok(Self::get_primitive_type(TypeKind::BYTE)),
            TypeIdentifier::TkInt8Type => Ok(Self::get_primitive_type(TypeKind::INT8)),
            TypeIdentifier::TkInt16Type => Ok(Self::get_primitive_type(TypeKind::INT16)),
            TypeIdentifier::TkInt32Type => Ok(Self::get_primitive_type(TypeKind::INT32)),
            TypeIdentifier::TkInt64Type => Ok(Self::get_primitive_type(TypeKind::INT64)),
            TypeIdentifier::TkUint8Type => Ok(Self::get_primitive_type(TypeKind::UINT8)),
            TypeIdentifier::TkUint16Type => Ok(Self::get_primitive_type(TypeKind::UINT16)),
            TypeIdentifier::TkUint32Type => Ok(Self::get_primitive_type(TypeKind::UINT32)),
            TypeIdentifier::TkUint64Type => Ok(Self::get_primitive_type(TypeKind::UINT64)),
            TypeIdentifier::TkFloat32Type => Ok(Self::get_primitive_type(TypeKind::FLOAT32)),
            TypeIdentifier::TkFloat64Type => Ok(Self::get_primitive_type(TypeKind::FLOAT64)),
            TypeIdentifier::TkFloat128Type => Ok(Self::get_primitive_type(TypeKind::FLOAT128)),
            TypeIdentifier::TkChar8Type => Ok(Self::get_primitive_type(TypeKind::CHAR8)),
            TypeIdentifier::TkChar16Type => Ok(Self::get_primitive_type(TypeKind::CHAR16)),

            // String types
            TypeIdentifier::TiString8Small { string_sdefn } => {
                Ok(Self::create_string_type(string_sdefn.bound as u32).build())
            }
            TypeIdentifier::TiString8Large { string_ldefn } => {
                Ok(Self::create_string_type(string_ldefn.bound).build())
            }
            TypeIdentifier::TiString16Small { .. } | TypeIdentifier::TiString16Large { .. } => {
                // Wide strings not fully supported
                Err(XTypesError::IllegalOperation)
            }

            // Sequence types
            TypeIdentifier::TiPlainSequenceSmall { seq_sdefn } => {
                let element_type =
                    Self::type_identifier_to_dynamic_type(&seq_sdefn.element_identifier)?;
                let bound = if seq_sdefn.bound == 0 {
                    INVALID_LBOUND // Unbounded
                } else {
                    seq_sdefn.bound as u32
                };
                Ok(Self::create_sequence_type(element_type, bound).build())
            }
            TypeIdentifier::TiPlainSequenceLarge { seq_ldefn } => {
                let element_type =
                    Self::type_identifier_to_dynamic_type(&seq_ldefn.element_identifier)?;
                Ok(Self::create_sequence_type(element_type, seq_ldefn.bound).build())
            }

            // Array types
            TypeIdentifier::TiPlainArraySmall { array_sdefn } => {
                let element_type =
                    Self::type_identifier_to_dynamic_type(&array_sdefn.element_identifier)?;
                let bounds: Vec<u32> = array_sdefn
                    .array_bound_seq
                    .iter()
                    .map(|&b| b as u32)
                    .collect();
                Ok(Self::create_array_type(element_type, bounds).build())
            }
            TypeIdentifier::TiPlainArrayLarge { array_ldefn } => {
                let element_type =
                    Self::type_identifier_to_dynamic_type(&array_ldefn.element_identifier)?;
                Ok(Self::create_array_type(element_type, array_ldefn.array_bound_seq.clone()).build())
            }

            // Map types - not fully implemented
            TypeIdentifier::TiPlainMapSmall { .. } | TypeIdentifier::TiPlainMapLarge { .. } => {
                Err(XTypesError::IllegalOperation)
            }

            // Strongly connected component - requires external resolution
            TypeIdentifier::TiStronglyConnectedComponent { .. } => {
                Err(XTypesError::IllegalOperation)
            }

            // Complete type reference - recursively convert
            TypeIdentifier::EkComplete { complete } => Ok(complete.as_ref().clone()),

            // Minimal type reference - cannot fully convert without name info
            TypeIdentifier::EkMinimal { .. } => Err(XTypesError::IllegalOperation),

            // Hash-only variants - cannot convert without looking up the full type
            TypeIdentifier::EkCompleteHash { .. } | TypeIdentifier::EkMinimalHash { .. } => {
                Err(XTypesError::IllegalOperation)
            }
        }
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
    pub default_value: Option<DataStorage>,
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
            .ok_or(XTypesError::InvalidIndex(index))
    }

    // fn get_annotation_count(&self) -> u32;
    // DDS::ReturnCode_t get_annotation(inout AnnotationDescriptor descriptor, in unsigned long idx);
    // unsigned long get_verbatim_text_count();
    // DDS::ReturnCode_t get_verbatim_text(inout VerbatimTextDescriptor descriptor, in unsigned long idx);

    pub(crate) fn clear_nonkey_members(&mut self) {
        self.member_list.retain(|m| m.descriptor.is_key);
        for m in self.member_list.iter_mut() {
            m.descriptor.r#type.clear_nonkey_members();
        }
    }

    /// Converts this DynamicType to a CompleteTypeObject for TypeLookup serialization.
    ///
    /// This is used when responding to TypeLookup requests to provide the full type
    /// information to remote participants.
    pub fn to_type_object(&self) -> XTypesResult<TypeObject> {
        match self.descriptor.kind {
            TypeKind::STRUCTURE => Ok(TypeObject::EkComplete {
                complete: self.to_complete_struct_type()?,
            }),
            // Other type kinds can be added as needed
            _ => Err(XTypesError::IllegalOperation),
        }
    }

    /// Converts this DynamicType to a CompleteStructType.
    fn to_complete_struct_type(&self) -> XTypesResult<CompleteTypeObject> {
        let struct_flags = StructTypeFlag {
            is_final: self.descriptor.extensibility_kind == ExtensibilityKind::Final,
            is_appendable: self.descriptor.extensibility_kind == ExtensibilityKind::Appendable,
            is_mutable: self.descriptor.extensibility_kind == ExtensibilityKind::Mutable,
            is_nested: self.descriptor.is_nested,
            is_autoid_hash: false, // Not using auto hash IDs
        };

        let header = CompleteStructHeader {
            base_type: TypeIdentifier::TkNone, // No inheritance for now
            detail: CompleteTypeDetail {
                ann_builtin: None,
                ann_custom: None,
                type_name: self.descriptor.name.clone(),
            },
        };

        let mut member_seq = Vec::new();
        for member in &self.member_list {
            let member_type_id = Self::dynamic_type_to_type_identifier(&member.descriptor.r#type)?;
            let member_flags = StructMemberFlag {
                try_construct: member.descriptor.try_construct_kind,
                is_external: false,
                is_optional: member.descriptor.is_optional,
                is_must_undestand: member.descriptor.is_must_understand,
                is_key: member.descriptor.is_key,
            };

            member_seq.push(CompleteStructMember {
                common: CommonStructMember {
                    member_id: member.descriptor.id,
                    member_flags,
                    member_type_id,
                },
                detail: CompleteMemberDetail {
                    name: member.descriptor.name.clone(),
                    ann_builtin: None,
                    ann_custom: None,
                },
            });
        }

        Ok(CompleteTypeObject::TkStructure {
            struct_type: CompleteStructType {
                struct_flags,
                header,
                member_seq,
            },
        })
    }

    /// Converts a DynamicType to a TypeIdentifier for use in member types.
    pub fn dynamic_type_to_type_identifier(dynamic_type: &DynamicType) -> XTypesResult<TypeIdentifier> {
        match dynamic_type.descriptor.kind {
            // Primitive types
            TypeKind::NONE => Ok(TypeIdentifier::TkNone),
            TypeKind::BOOLEAN => Ok(TypeIdentifier::TkBoolean),
            TypeKind::BYTE => Ok(TypeIdentifier::TkByteType),
            TypeKind::INT8 => Ok(TypeIdentifier::TkInt8Type),
            TypeKind::INT16 => Ok(TypeIdentifier::TkInt16Type),
            TypeKind::INT32 => Ok(TypeIdentifier::TkInt32Type),
            TypeKind::INT64 => Ok(TypeIdentifier::TkInt64Type),
            TypeKind::UINT8 => Ok(TypeIdentifier::TkUint8Type),
            TypeKind::UINT16 => Ok(TypeIdentifier::TkUint16Type),
            TypeKind::UINT32 => Ok(TypeIdentifier::TkUint32Type),
            TypeKind::UINT64 => Ok(TypeIdentifier::TkUint64Type),
            TypeKind::FLOAT32 => Ok(TypeIdentifier::TkFloat32Type),
            TypeKind::FLOAT64 => Ok(TypeIdentifier::TkFloat64Type),
            TypeKind::FLOAT128 => Ok(TypeIdentifier::TkFloat128Type),
            TypeKind::CHAR8 => Ok(TypeIdentifier::TkChar8Type),
            TypeKind::CHAR16 => Ok(TypeIdentifier::TkChar16Type),

            // String type - use large bound for unbounded strings
            TypeKind::STRING8 => {
                let bound = dynamic_type.descriptor.bound.first().copied().unwrap_or(0);
                Ok(TypeIdentifier::TiString8Large {
                    string_ldefn: StringLTypeDefn { bound },
                })
            }

            // Sequence type
            TypeKind::SEQUENCE => {
                let element_type = dynamic_type
                    .descriptor
                    .element_type
                    .as_ref()
                    .ok_or(XTypesError::InvalidData)?;
                let element_identifier = Self::dynamic_type_to_type_identifier(element_type)?;
                let bound = dynamic_type.descriptor.bound.first().copied().unwrap_or(0);

                Ok(TypeIdentifier::TiPlainSequenceLarge {
                    seq_ldefn: Box::new(PlainSequenceLElemDefn {
                        header: PlainCollectionHeader {
                            equiv_kind: EK_COMPLETE,
                            element_flags: CollectionElementFlag {
                                try_construct: TryConstructKind::UseDefault,
                                is_external: false,
                            },
                        },
                        bound,
                        element_identifier,
                    }),
                })
            }

            // Nested structure - embed as EkComplete with the full DynamicType
            TypeKind::STRUCTURE => Ok(TypeIdentifier::EkComplete {
                complete: Box::new(dynamic_type.clone()),
            }),

            // Other types not yet supported for TypeLookup
            _ => Err(XTypesError::IllegalOperation),
        }
    }

    /// Generates TypeInformation bytes for this DynamicType.
    ///
    /// TypeInformation is used in endpoint discovery (PID_TYPE_INFORMATION) to
    /// communicate type identity to remote participants. This allows them to
    /// send TypeLookup requests to get the full TypeObject.
    ///
    /// The generated TypeIdentifier uses the equivalence hash format (EK_COMPLETE)
    /// with an MD5 hash of the serialized TypeObject.
    pub fn to_type_information_bytes(&self) -> Vec<u8> {
        // Compute equivalence hash from the type
        let hash = self.compute_equivalence_hash();

        // Create TypeIdentifier with the hash (not the full DynamicType)
        // This is the proper format for discovery - just the 14-byte hash
        let mut type_id_bytes = Vec::with_capacity(15);
        type_id_bytes.push(EK_COMPLETE);
        type_id_bytes.extend_from_slice(&hash);

        // Create TypeIdentifierWithSize with raw bytes
        // We serialize manually to avoid circular issues with EkComplete containing DynamicType
        let mut typeid_with_size_bytes = type_id_bytes.clone();
        typeid_with_size_bytes.extend_from_slice(&0u32.to_le_bytes()); // typeobject_serialized_size

        // Create TypeIdentifierWithDependencies (APPENDABLE with DHEADER)
        let mut type_with_deps_bytes = Vec::new();
        // dependent_typeid_count: i32
        type_with_deps_bytes.extend_from_slice(&typeid_with_size_bytes);
        type_with_deps_bytes.extend_from_slice(&0i32.to_le_bytes()); // dependent_typeid_count
        type_with_deps_bytes.extend_from_slice(&0u32.to_le_bytes()); // dependent_typeids length

        // Prepend DHEADER
        let mut type_with_deps_final = Vec::with_capacity(4 + type_with_deps_bytes.len());
        type_with_deps_final.extend_from_slice(&(type_with_deps_bytes.len() as u32).to_le_bytes());
        type_with_deps_final.extend_from_slice(&type_with_deps_bytes);

        // Create TypeInformation (MUTABLE with parameter list)
        let mut inner = Vec::new();

        // minimal field (PID 0x1001)
        inner.extend_from_slice(&0x1001u32.to_le_bytes()); // PID
        inner.extend_from_slice(&(type_with_deps_final.len() as u32).to_le_bytes()); // length
        inner.extend_from_slice(&type_with_deps_final);

        // complete field (PID 0x1002) - same data
        inner.extend_from_slice(&0x1002u32.to_le_bytes()); // PID
        inner.extend_from_slice(&(type_with_deps_final.len() as u32).to_le_bytes()); // length
        inner.extend_from_slice(&type_with_deps_final);

        // Sentinel (PID 0x0001)
        inner.extend_from_slice(&0x0001u32.to_le_bytes()); // PID_SENTINEL
        inner.extend_from_slice(&0u32.to_le_bytes()); // length 0

        // Prepend DHEADER for TypeInformation
        let mut result = Vec::with_capacity(4 + inner.len());
        result.extend_from_slice(&(inner.len() as u32).to_le_bytes());
        result.extend_from_slice(&inner);
        result
    }

    /// Computes the 14-byte equivalence hash for this type.
    /// Per XTypes spec, this is MD5(serialized TypeObject) truncated to 14 bytes.
    /// Falls back to MD5 of type name if TypeObject serialization fails.
    pub fn compute_equivalence_hash(&self) -> [u8; 14] {
        // Try to serialize the TypeObject for a proper hash
        if let Ok(type_object) = self.to_type_object() {
            let serialized = type_object.serialize_to_bytes();
            let digest: [u8; 16] = md5::compute(&serialized).into();
            let mut hash = [0u8; 14];
            hash.copy_from_slice(&digest[..14]);
            return hash;
        }

        // Fallback: hash the type name
        let mut context = md5::Context::new();
        context.consume(self.get_name().as_bytes());
        let digest: [u8; 16] = context.compute().into();
        let mut hash = [0u8; 14];
        hash.copy_from_slice(&digest[..14]);
        hash
    }

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
    abstract_data: BTreeMap<MemberId, DataStorage>,
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
            .ok_or(XTypesError::InvalidId(id))
    }

    pub fn set_descriptor(&mut self, _id: MemberId, _value: MemberDescriptor) -> XTypesResult<()> {
        todo!()
    }

    pub fn get_member_id_by_name(&self, name: &str) -> Option<MemberId> {
        self.type_ref
            .member_list
            .iter()
            .find(|m| m.get_name() == name)
            .map(|m| m.get_id())
    }

    /// Returns a reference to the raw value storage for the member with the given name.
    ///
    /// This combines name lookup with value access for convenience.
    pub fn get_value_by_name(&self, name: &str) -> XTypesResult<&DataStorage> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_value(id)
    }

    /// Returns an i32 value for the member with the given name.
    pub fn get_int32_value_by_name(&self, name: &str) -> XTypesResult<&i32> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_int32_value(id)
    }

    /// Returns a u32 value for the member with the given name.
    pub fn get_uint32_value_by_name(&self, name: &str) -> XTypesResult<&u32> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_uint32_value(id)
    }

    /// Returns a string value for the member with the given name.
    pub fn get_string_value_by_name(&self, name: &str) -> XTypesResult<&String> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_string_value(id)
    }

    /// Returns a bool value for the member with the given name.
    pub fn get_boolean_value_by_name(&self, name: &str) -> XTypesResult<&bool> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_boolean_value(id)
    }

    /// Returns a nested DynamicData value for the member with the given name.
    pub fn get_complex_value_by_name(&self, name: &str) -> XTypesResult<&DynamicData> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_complex_value(id)
    }

    /// Returns a u8 value for the member with the given name.
    pub fn get_uint8_value_by_name(&self, name: &str) -> XTypesResult<&u8> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_uint8_value(id)
    }

    /// Returns a char value for the member with the given name.
    pub fn get_char8_value_by_name(&self, name: &str) -> XTypesResult<&char> {
        let id = self
            .get_member_id_by_name(name)
            .ok_or(XTypesError::InvalidId(0))?;
        self.get_char8_value(id)
    }

    pub fn get_member_id_at_index(&self, index: u32) -> XTypesResult<MemberId> {
        self.abstract_data
            .keys()
            .nth(index as usize)
            .cloned()
            .ok_or(XTypesError::InvalidIndex(index))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xtypes::type_object::{
        CommonStructMember, CompleteMemberDetail, CompleteStructHeader, CompleteStructMember,
        CompleteTypeDetail, StructMemberFlag, StructTypeFlag,
    };

    /// Test converting a simple struct TypeObject to DynamicType.
    ///
    /// This simulates receiving a HelloWorldType via TypeLookup:
    /// ```idl
    /// struct HelloWorldType {
    ///     octet id;
    ///     char msg;
    /// };
    /// ```
    #[test]
    fn test_complete_struct_to_dynamic_type() {
        // Create a CompleteStructType similar to HelloWorldType
        let struct_type = CompleteStructType {
            struct_flags: StructTypeFlag {
                is_final: true,
                is_appendable: false,
                is_mutable: false,
                is_nested: false,
                is_autoid_hash: false,
            },
            header: CompleteStructHeader {
                base_type: TypeIdentifier::TkNone,
                detail: CompleteTypeDetail {
                    ann_builtin: None,
                    ann_custom: None,
                    type_name: String::from("test::HelloWorldType"),
                },
            },
            member_seq: vec![
                CompleteStructMember {
                    common: CommonStructMember {
                        member_id: 0,
                        member_flags: StructMemberFlag {
                            try_construct: TryConstructKind::UseDefault,
                            is_external: false,
                            is_optional: false,
                            is_must_undestand: false,
                            is_key: false,
                        },
                        member_type_id: TypeIdentifier::TkUint8Type,
                    },
                    detail: CompleteMemberDetail {
                        name: String::from("id"),
                        ann_builtin: None,
                        ann_custom: None,
                    },
                },
                CompleteStructMember {
                    common: CommonStructMember {
                        member_id: 1,
                        member_flags: StructMemberFlag {
                            try_construct: TryConstructKind::UseDefault,
                            is_external: false,
                            is_optional: false,
                            is_must_undestand: false,
                            is_key: false,
                        },
                        member_type_id: TypeIdentifier::TkChar8Type,
                    },
                    detail: CompleteMemberDetail {
                        name: String::from("msg"),
                        ann_builtin: None,
                        ann_custom: None,
                    },
                },
            ],
        };

        let type_object = TypeObject::EkComplete {
            complete: CompleteTypeObject::TkStructure { struct_type },
        };

        // Convert TypeObject to DynamicType
        let result = DynamicTypeBuilderFactory::create_type_w_type_object(type_object);
        assert!(result.is_ok(), "Failed to convert TypeObject: {:?}", result);

        let dynamic_type = result.unwrap();

        // Verify type properties
        assert_eq!(dynamic_type.get_name(), "test::HelloWorldType");
        assert_eq!(dynamic_type.get_kind(), TypeKind::STRUCTURE);
        assert_eq!(
            dynamic_type.get_descriptor().extensibility_kind,
            ExtensibilityKind::Final
        );
        assert_eq!(dynamic_type.get_member_count(), 2);

        // Verify first member (id: octet)
        let member0 = dynamic_type.get_member_by_index(0).unwrap();
        assert_eq!(member0.get_name(), "id");
        assert_eq!(member0.get_id(), 0);
        let desc0 = member0.get_descriptor().unwrap();
        assert_eq!(desc0.r#type.get_kind(), TypeKind::UINT8);

        // Verify second member (msg: char)
        let member1 = dynamic_type.get_member_by_index(1).unwrap();
        assert_eq!(member1.get_name(), "msg");
        assert_eq!(member1.get_id(), 1);
        let desc1 = member1.get_descriptor().unwrap();
        assert_eq!(desc1.r#type.get_kind(), TypeKind::CHAR8);
    }

    #[test]
    fn test_type_identifier_to_dynamic_type_primitives() {
        // Test primitive type conversions
        let result =
            DynamicTypeBuilderFactory::type_identifier_to_dynamic_type(&TypeIdentifier::TkBoolean);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_kind(), TypeKind::BOOLEAN);

        let result =
            DynamicTypeBuilderFactory::type_identifier_to_dynamic_type(&TypeIdentifier::TkInt32Type);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_kind(), TypeKind::INT32);

        let result =
            DynamicTypeBuilderFactory::type_identifier_to_dynamic_type(&TypeIdentifier::TkUint8Type);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_kind(), TypeKind::UINT8);

        let result = DynamicTypeBuilderFactory::type_identifier_to_dynamic_type(
            &TypeIdentifier::TkFloat64Type,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_kind(), TypeKind::FLOAT64);

        let result =
            DynamicTypeBuilderFactory::type_identifier_to_dynamic_type(&TypeIdentifier::TkChar8Type);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_kind(), TypeKind::CHAR8);
    }

    #[test]
    fn test_minimal_type_object_returns_error() {
        // MinimalTypeObject cannot be fully converted because it doesn't contain member names
        let type_object = TypeObject::EkMinimal {
            minimal: crate::xtypes::type_object::MinimalTypeObject::TkStructure {
                struct_type: crate::xtypes::type_object::MinimalStructType {
                    struct_flags: StructTypeFlag {
                        is_final: true,
                        is_appendable: false,
                        is_mutable: false,
                        is_nested: false,
                        is_autoid_hash: false,
                    },
                    header: crate::xtypes::type_object::MinimalStructHeader {
                        base_type: TypeIdentifier::TkNone,
                        detail: crate::xtypes::type_object::MinimalTypeDetail {},
                    },
                    member_seq: vec![],
                },
            },
        };

        let result = DynamicTypeBuilderFactory::create_type_w_type_object(type_object);
        assert!(
            result.is_err(),
            "Should return error for MinimalTypeObject"
        );
    }
}
