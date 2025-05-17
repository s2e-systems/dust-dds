use crate::xtypes::{
    dynamic_type::{
        DynamicType, DynamicTypeMember, ExtensibilityKind, MemberDescriptor, MemberId, ObjectName,
        TryConstructKind, TypeDescriptor,
    },
    error::XTypesError,
    type_object::{
        CompleteAnnotationParameter, CompleteBitfield, CompleteBitflag, CompleteEnumeratedLiteral,
        CompleteStructMember, CompleteTypeObject, CompleteUnionMember, TypeIdentifier, TypeKind,
        TK_ALIAS, TK_ANNOTATION, TK_ARRAY, TK_BITMASK, TK_BITSET, TK_BOOLEAN, TK_BYTE, TK_CHAR16,
        TK_CHAR8, TK_ENUM, TK_FLOAT128, TK_FLOAT32, TK_FLOAT64, TK_INT16, TK_INT32, TK_INT64,
        TK_INT8, TK_MAP, TK_NONE, TK_SEQUENCE, TK_STRING16, TK_STRING8, TK_STRUCTURE, TK_UINT16,
        TK_UINT32, TK_UINT64, TK_UINT8, TK_UNION,
    },
};
use alloc::string::String;

impl DynamicType for TypeIdentifier {
    fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => Ok(TypeDescriptor {
                kind: self.get_kind(),
                name: self.get_name(),
                extensibility_kind: ExtensibilityKind::Final,
                is_nested: false,
            }),
            TypeIdentifier::TiStronglyConnectedComponent { .. } => unimplemented!(),
            TypeIdentifier::EkComplete { complete } => complete.get_descriptor(),
            TypeIdentifier::EkMinimal { .. } => unimplemented!(),
        }
    }

    fn get_name(&self) -> ObjectName {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => String::new(),
            TypeIdentifier::TiStronglyConnectedComponent { .. } => unimplemented!(),
            TypeIdentifier::EkComplete { complete } => complete.get_name(),
            TypeIdentifier::EkMinimal { .. } => unimplemented!(),
        }
    }

    fn get_kind(&self) -> TypeKind {
        match self {
            TypeIdentifier::TkNone => TK_NONE,
            TypeIdentifier::TkBoolean => TK_BOOLEAN,
            TypeIdentifier::TkByteType => TK_BYTE,
            TypeIdentifier::TkInt8Type => TK_INT8,
            TypeIdentifier::TkInt16Type => TK_INT16,
            TypeIdentifier::TkInt32Type => TK_INT32,
            TypeIdentifier::TkInt64Type => TK_INT64,
            TypeIdentifier::TkUint8Type => TK_UINT8,
            TypeIdentifier::TkUint16Type => TK_UINT16,
            TypeIdentifier::TkUint32Type => TK_UINT32,
            TypeIdentifier::TkUint64Type => TK_UINT64,
            TypeIdentifier::TkFloat32Type => TK_FLOAT32,
            TypeIdentifier::TkFloat64Type => TK_FLOAT64,
            TypeIdentifier::TkFloat128Type => TK_FLOAT128,
            TypeIdentifier::TkChar8Type => TK_CHAR8,
            TypeIdentifier::TkChar16Type => TK_CHAR16,
            TypeIdentifier::TiString8Small { .. } | TypeIdentifier::TiString8Large { .. } => {
                TK_STRING8
            }
            TypeIdentifier::TiString16Small { .. } | TypeIdentifier::TiString16Large { .. } => {
                TK_STRING16
            }
            TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. } => TK_SEQUENCE,
            TypeIdentifier::TiPlainArraySmall { .. } | TypeIdentifier::TiPlainArrayLarge { .. } => {
                TK_ARRAY
            }
            TypeIdentifier::TiPlainMapSmall { .. } | TypeIdentifier::TiPlainMapLarge { .. } => {
                TK_MAP
            }
            TypeIdentifier::TiStronglyConnectedComponent { .. } => unimplemented!(),
            TypeIdentifier::EkComplete { complete } => complete.get_kind(),
            TypeIdentifier::EkMinimal { .. } => unimplemented!(),
        }
    }

    fn get_member_count(&self) -> u32 {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => 0,
            TypeIdentifier::TiStronglyConnectedComponent { .. } => unimplemented!(),
            TypeIdentifier::EkComplete { complete } => complete.get_member_count(),
            TypeIdentifier::EkMinimal { .. } => unimplemented!(),
        }
    }

    fn get_member_by_index(&self, index: u32) -> Result<&dyn DynamicTypeMember, XTypesError> {
        match self {
            TypeIdentifier::TkNone
            | TypeIdentifier::TkBoolean
            | TypeIdentifier::TkByteType
            | TypeIdentifier::TkInt8Type
            | TypeIdentifier::TkInt16Type
            | TypeIdentifier::TkInt32Type
            | TypeIdentifier::TkInt64Type
            | TypeIdentifier::TkUint8Type
            | TypeIdentifier::TkUint16Type
            | TypeIdentifier::TkUint32Type
            | TypeIdentifier::TkUint64Type
            | TypeIdentifier::TkFloat32Type
            | TypeIdentifier::TkFloat64Type
            | TypeIdentifier::TkFloat128Type
            | TypeIdentifier::TkChar8Type
            | TypeIdentifier::TkChar16Type
            | TypeIdentifier::TiString8Small { .. }
            | TypeIdentifier::TiString16Small { .. }
            | TypeIdentifier::TiString8Large { .. }
            | TypeIdentifier::TiString16Large { .. }
            | TypeIdentifier::TiPlainSequenceSmall { .. }
            | TypeIdentifier::TiPlainSequenceLarge { .. }
            | TypeIdentifier::TiPlainArraySmall { .. }
            | TypeIdentifier::TiPlainArrayLarge { .. }
            | TypeIdentifier::TiPlainMapSmall { .. }
            | TypeIdentifier::TiPlainMapLarge { .. } => Err(XTypesError::InvalidIndex),
            TypeIdentifier::TiStronglyConnectedComponent { .. } => unimplemented!(),
            TypeIdentifier::EkComplete { complete } => complete.get_member_by_index(index),
            TypeIdentifier::EkMinimal { .. } => unimplemented!(),
        }
    }
}

impl DynamicType for CompleteTypeObject {
    fn get_descriptor(&self) -> Result<TypeDescriptor, XTypesError> {
        Ok(TypeDescriptor {
            kind: self.get_kind(),
            name: self.get_name(),
            extensibility_kind: {
                match self {
                    CompleteTypeObject::TkAlias { .. }
                    | CompleteTypeObject::TkAnnotation { .. }
                    | CompleteTypeObject::TkBitset { .. }
                    | CompleteTypeObject::TkBitmask { .. }
                    | CompleteTypeObject::TkSequence { .. }
                    | CompleteTypeObject::TkArray { .. }
                    | CompleteTypeObject::TkMap { .. }
                    | CompleteTypeObject::TkEnum { .. } => ExtensibilityKind::Final,
                    CompleteTypeObject::TkStructure { struct_type } => {
                        if struct_type.struct_flags.is_final {
                            ExtensibilityKind::Final
                        } else if struct_type.struct_flags.is_appendable {
                            ExtensibilityKind::Appendable
                        } else {
                            ExtensibilityKind::Mutable
                        }
                    }
                    CompleteTypeObject::TkUnion { union_type } => {
                        if union_type.union_flags.is_final {
                            ExtensibilityKind::Final
                        } else if union_type.union_flags.is_appendable {
                            ExtensibilityKind::Appendable
                        } else {
                            ExtensibilityKind::Mutable
                        }
                    }
                }
            },
            is_nested: false,
        })
    }

    fn get_name(&self) -> ObjectName {
        match self {
            CompleteTypeObject::TkAlias { alias_type } => {
                alias_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkAnnotation { annotation_type } => {
                annotation_type.header.annotation_name.clone()
            }
            CompleteTypeObject::TkStructure { struct_type } => {
                struct_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkUnion { union_type } => {
                union_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkBitset { bitset_type } => {
                bitset_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkSequence { .. }
            | CompleteTypeObject::TkArray { .. }
            | CompleteTypeObject::TkMap { .. } => String::new(),
            CompleteTypeObject::TkEnum { enumerated_type } => {
                enumerated_type.header.detail.type_name.clone()
            }
            CompleteTypeObject::TkBitmask { bitmask_type } => {
                bitmask_type.header.detail.type_name.clone()
            }
        }
    }

    fn get_kind(&self) -> TypeKind {
        match self {
            CompleteTypeObject::TkAlias { .. } => TK_ALIAS,
            CompleteTypeObject::TkAnnotation { .. } => TK_ANNOTATION,
            CompleteTypeObject::TkStructure { .. } => TK_STRUCTURE,
            CompleteTypeObject::TkUnion { .. } => TK_UNION,
            CompleteTypeObject::TkBitset { .. } => TK_BITSET,
            CompleteTypeObject::TkSequence { .. } => TK_SEQUENCE,
            CompleteTypeObject::TkArray { .. } => TK_ARRAY,
            CompleteTypeObject::TkMap { .. } => TK_MAP,
            CompleteTypeObject::TkEnum { .. } => TK_ENUM,
            CompleteTypeObject::TkBitmask { .. } => TK_BITMASK,
        }
    }

    fn get_member_count(&self) -> u32 {
        match self {
            CompleteTypeObject::TkAlias { .. } => 0,
            CompleteTypeObject::TkAnnotation { annotation_type } => {
                annotation_type.member_seq.len() as u32
            }
            CompleteTypeObject::TkStructure { struct_type } => struct_type.member_seq.len() as u32,
            CompleteTypeObject::TkUnion { union_type } => union_type.member_seq.len() as u32,
            CompleteTypeObject::TkBitset { bitset_type } => bitset_type.field_seq.len() as u32,
            CompleteTypeObject::TkSequence { .. }
            | CompleteTypeObject::TkArray { .. }
            | CompleteTypeObject::TkMap { .. } => 0,
            CompleteTypeObject::TkEnum { enumerated_type } => {
                enumerated_type.literal_seq.len() as u32
            }
            CompleteTypeObject::TkBitmask { bitmask_type } => bitmask_type.flag_seq.len() as u32,
        }
    }

    fn get_member_by_index(
        &self,
        index: u32,
    ) -> Result<&dyn crate::xtypes::dynamic_type::DynamicTypeMember, XTypesError> {
        match self {
            CompleteTypeObject::TkAlias { .. } => Err(XTypesError::InvalidIndex),
            CompleteTypeObject::TkAnnotation { annotation_type } => Ok(annotation_type
                .member_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkStructure { struct_type } => Ok(struct_type
                .member_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkUnion { union_type } => Ok(union_type
                .member_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkBitset { bitset_type } => Ok(bitset_type
                .field_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkSequence { .. }
            | CompleteTypeObject::TkArray { .. }
            | CompleteTypeObject::TkMap { .. } => Err(XTypesError::InvalidIndex),
            CompleteTypeObject::TkEnum { enumerated_type } => Ok(enumerated_type
                .literal_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
            CompleteTypeObject::TkBitmask { bitmask_type } => Ok(bitmask_type
                .flag_seq
                .get(index as usize)
                .ok_or(XTypesError::InvalidIndex)?),
        }
    }
}

impl DynamicTypeMember for CompleteStructMember {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            type_: &self.common.member_type_id,
            default_value: "",
            index: 0,
            try_construct_kind: TryConstructKind::Discard,
            is_key: self.common.member_flags.is_key,
            is_optional: self.common.member_flags.is_optional,
            is_must_understand: self.common.member_flags.is_must_undestand,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        self.common.member_id
    }

    fn get_name(&self) -> ObjectName {
        self.detail.name.clone()
    }
}

impl DynamicTypeMember for CompleteEnumeratedLiteral {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            type_: &TypeIdentifier::TkNone,
            default_value: "",
            index: 0,
            try_construct_kind: TryConstructKind::Discard,
            is_key: false,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        0
    }

    fn get_name(&self) -> ObjectName {
        String::new()
    }
}

impl DynamicTypeMember for CompleteUnionMember {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            type_: &self.common.type_id,
            default_value: "",
            index: 0,
            try_construct_kind: self.common.member_flags.try_construct,
            is_key: false,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        self.common.member_id
    }

    fn get_name(&self) -> ObjectName {
        self.detail.name.clone()
    }
}

impl DynamicTypeMember for CompleteAnnotationParameter {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: 0,
            type_: &self.common.member_type_id,
            default_value: "",
            index: 0,
            try_construct_kind: TryConstructKind::Discard,
            is_key: false,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        0
    }

    fn get_name(&self) -> ObjectName {
        self.name.clone()
    }
}

impl DynamicTypeMember for CompleteBitfield {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            type_: &TypeIdentifier::TkNone,
            default_value: "",
            index: self.common.position as u32,
            try_construct_kind: TryConstructKind::Discard,
            is_key: false,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        self.common.position as u32
    }

    fn get_name(&self) -> ObjectName {
        self.detail.name.clone()
    }
}

impl DynamicTypeMember for CompleteBitflag {
    fn get_descriptor(&self) -> Result<MemberDescriptor, XTypesError> {
        Ok(MemberDescriptor {
            name: self.get_name(),
            id: self.get_id(),
            type_: &TypeIdentifier::TkNone,
            default_value: "",
            index: self.common.position as u32,
            try_construct_kind: TryConstructKind::Discard,
            is_key: false,
            is_optional: false,
            is_must_understand: true,
            is_shared: false,
            is_default_label: false,
        })
    }

    fn get_id(&self) -> MemberId {
        self.common.position as u32
    }

    fn get_name(&self) -> ObjectName {
        self.detail.name.clone()
    }
}
