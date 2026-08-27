use crate::{
    infrastructure::instance::InstanceHandle,
    transport::types::TopicKind,
    xtypes::{
        dynamic_type::{
            DynamicData, DynamicDataFactory, DynamicType, DynamicTypeMember, TypeDescriptor,
            TypeKind,
        },
        error::{XTypesError, XTypesResult},
        serializer::serialize_final_without_header,
    },
};
use alloc::{boxed::Box, vec::Vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyHolderType {
    descriptor: Option<TypeDescriptor>,
    key_members: Box<[DynamicTypeMember]>,
}

impl Default for KeyHolderType {
    fn default() -> Self {
        Self {
            descriptor: None,
            key_members: Box::new([]),
        }
    }
}

impl KeyHolderType {
    pub fn new(value: &DynamicType<'_>) -> Self {
        let mut member_list = Vec::new();
        fn fill_struct_key_holder_type(
            value: &DynamicType<'_>,
            member_list: &mut Vec<DynamicTypeMember>,
        ) {
            if value.get_kind() == TypeKind::STRUCTURE {
                for member in value.member_list {
                    if member.descriptor.is_key {
                        member_list.push(member.clone());
                    } else if member.descriptor.r#type.descriptor.kind == TypeKind::STRUCTURE
                        && !member.descriptor.is_optional
                    {
                        fill_struct_key_holder_type(&member.descriptor.r#type, member_list);
                    }
                }
            }
        }

        fill_struct_key_holder_type(value, &mut member_list);
        Self {
            descriptor: Some(value.descriptor.clone()),
            key_members: member_list.into_boxed_slice(),
        }
    }

    pub fn as_dynamic_type(&self) -> Option<DynamicType<'_>> {
        self.descriptor.as_ref().map(|descriptor| DynamicType {
            descriptor,
            member_list: &self.key_members,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.key_members.is_empty()
    }
}

impl From<&DynamicType<'_>> for TopicKind {
    fn from(value: &DynamicType<'_>) -> Self {
        if value.get_kind() == TypeKind::STRUCTURE {
            for member in value.member_list {
                if member.descriptor.is_key {
                    return TopicKind::WithKey;
                } else if member.descriptor.r#type.descriptor.kind == TypeKind::STRUCTURE
                    && !member.descriptor.is_optional
                {
                    let member_topic_kind = TopicKind::from(&member.descriptor.r#type);
                    if member_topic_kind == TopicKind::WithKey {
                        return TopicKind::WithKey;
                    }
                }
            }
        }
        TopicKind::NoKey
    }
}

pub struct KeyHolderData<'a>(DynamicData<'a>);

impl<'a> KeyHolderData<'a> {
    pub fn from_dynamic_data(
        value: &DynamicData<'a>,
        key_holder_type: &'a KeyHolderType,
    ) -> XTypesResult<KeyHolderData<'a>> {
        fn fill_struct_key_holder_data<'a>(
            value: &DynamicData<'a>,
            key_holder_data: &mut DynamicData,
        ) -> XTypesResult<()> {
            let dynamic_type = value.r#type();
            if dynamic_type.get_kind() == TypeKind::STRUCTURE {
                for member_index in 0..dynamic_type.get_member_count() {
                    let dynamic_type_member = dynamic_type.get_member_by_index(member_index)?;
                    let key_member_id = dynamic_type_member.get_id();
                    if dynamic_type_member.descriptor.is_key {
                        key_holder_data
                            .set_value(key_member_id, value.get_value(key_member_id)?.clone());
                    } else if dynamic_type_member.descriptor.r#type.get_kind()
                        == TypeKind::STRUCTURE
                        && !dynamic_type_member.descriptor.is_optional
                    {
                        fill_struct_key_holder_data(
                            value.get_complex_value(key_member_id)?,
                            key_holder_data,
                        )?;
                    }
                }
            }
            Ok(())
        }
        let dynamic_type = key_holder_type
            .as_dynamic_type()
            .ok_or(XTypesError::InvalidType)?;
        let mut key_holder_data = DynamicDataFactory::create_data(dynamic_type);
        fill_struct_key_holder_data(value, &mut key_holder_data)?;
        Ok(Self(key_holder_data))
    }

    pub fn as_dynamic_data(&self) -> &DynamicData<'a> {
        &self.0
    }
}

pub fn get_instance_handle_from_key_holder_data<'a>(
    key_holder_data: &KeyHolderData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let data = serialize_final_without_header(Vec::with_capacity(16), &key_holder_data.0)?;
    let key = if data.len() <= 16 {
        let mut key = [0; 16];
        key[0..data.len()].copy_from_slice(&data);
        key
    } else {
        md5::compute(data).into()
    };

    Ok(InstanceHandle::new(key))
}

pub fn get_instance_handle_from_dynamic_data_and_key_holder<'a>(
    value: &DynamicData<'a>,
    key_holder_type: &KeyHolderType,
) -> Result<InstanceHandle, XTypesError> {
    if key_holder_type.is_empty() {
        return Ok(InstanceHandle::new([0; 16]));
    }
    let key_holder_data = KeyHolderData::from_dynamic_data(value, key_holder_type)?;
    get_instance_handle_from_key_holder_data(&key_holder_data)
}

pub fn get_instance_handle_from_dynamic_data<'a>(
    value: &DynamicData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let key_holder_type = KeyHolderType::new(&value.r#type());
    get_instance_handle_from_dynamic_data_and_key_holder(value, &key_holder_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{infrastructure::instance::InstanceHandle, xtypes::type_support::TypeSupport};

    #[test]
    fn test_multiple_nested_keys() {
        #[derive(TypeSupport)]
        struct Inner {
            #[dust_dds(key)]
            id: u8,
        }

        #[derive(TypeSupport)]
        struct Outer {
            a: Inner,
            #[dust_dds(key)]
            b: u16,
        }

        #[derive(TypeSupport)]
        struct OuterOuter {
            a: Outer,
        }

        let outer_outer = OuterOuter {
            a: Outer {
                a: Inner { id: 1 },
                b: 3,
            },
        };

        let data = outer_outer.create_dynamic_sample();

        let instance_handle = get_instance_handle_from_dynamic_data(&data).unwrap();

        assert_eq!(
            instance_handle,
            InstanceHandle::new([1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        )
    }

    #[test]
    fn test_full_struct_key() {
        #[derive(TypeSupport)]
        struct Inner {
            id: u8,
            b: u16,
        }

        #[derive(TypeSupport)]
        struct Outer {
            #[dust_dds(key)]
            a: Inner,
        }

        let outer = Outer {
            a: Inner { id: 1, b: 3 },
        };

        let data = outer.create_dynamic_sample();

        let instance_handle = get_instance_handle_from_dynamic_data(&data).unwrap();

        assert_eq!(
            instance_handle,
            InstanceHandle::new([1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
        )
    }
}
