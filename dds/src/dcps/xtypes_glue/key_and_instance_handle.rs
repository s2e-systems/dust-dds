use crate::{
    infrastructure::instance::InstanceHandle,
    transport::types::TopicKind,
    xtypes::{
        dynamic_type::{
            DynamicData, DynamicDataFactory, DynamicType, DynamicTypeBuilder,
            DynamicTypeBuilderFactory, TypeKind,
        },
        error::{XTypesError, XTypesResult},
        serializer::serialize_final_without_header,
    },
};
use alloc::vec::Vec;

pub struct KeyHolderType<'a>(DynamicType<'a>);

impl<'a> KeyHolderType<'a> {
    pub fn from_dynamic_type(value: &DynamicType<'a>) -> XTypesResult<Self> {
        fn fill_struct_key_holder_type<'a>(
            value: &DynamicType<'a>,
            builder: &mut DynamicTypeBuilder,
        ) -> XTypesResult<()> {
            if value.get_kind() == TypeKind::STRUCTURE {
                for member_index in 0..value.get_member_count() {
                    let dynamic_type_member = value.get_member_by_index(member_index)?;
                    if dynamic_type_member.get_descriptor()?.is_key {
                        builder.add_member(dynamic_type_member.get_descriptor()?.clone())?;
                    } else if dynamic_type_member.descriptor.r#type.descriptor.kind
                        == TypeKind::STRUCTURE
                    {
                        fill_struct_key_holder_type(
                            &dynamic_type_member.descriptor.r#type,
                            builder,
                        )?;
                    }
                }
            }
            Ok(())
        }

        let mut key_holder_type_builder =
            DynamicTypeBuilderFactory::create_type(value.descriptor.clone());
        fill_struct_key_holder_type(value, &mut key_holder_type_builder)?;
        Ok(Self(key_holder_type_builder.build()))
    }

    pub fn get_topic_kind(&self) -> TopicKind {
        if self.0.member_list.is_empty() {
            TopicKind::NoKey
        } else {
            TopicKind::WithKey
        }
    }
}

pub struct KeyHolderData<'a>(DynamicData<'a>);

impl<'a> KeyHolderData<'a> {
    pub fn from_dynamic_data(value: &DynamicData<'a>) -> XTypesResult<KeyHolderData<'a>> {
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
        let key_holder_type = KeyHolderType::from_dynamic_type(&value.r#type())?.0;
        let mut key_holder_data = DynamicDataFactory::create_data(key_holder_type);
        fill_struct_key_holder_data(value, &mut key_holder_data)?;
        Ok(Self(key_holder_data))
    }

    pub fn as_dynamic_data(&self) -> &DynamicData<'a> {
        &self.0
    }

    pub fn get_topic_kind(&self) -> TopicKind {
        KeyHolderType(self.0.r#type()).get_topic_kind()
    }
}

pub fn get_instance_handle_from_key_holder_data<'a>(
    key_holder_data: &KeyHolderData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let data = serialize_final_without_header(Vec::new(), &key_holder_data.0)?;
    let key = if data.len() <= 16 {
        let mut key = [0; 16];
        key[0..data.len()].copy_from_slice(&data);
        key
    } else {
        md5::compute(data).into()
    };

    Ok(InstanceHandle::new(key))
}

pub fn get_instance_handle_from_dynamic_data<'a>(
    value: &DynamicData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let key_holder_data = KeyHolderData::from_dynamic_data(value)?;
    get_instance_handle_from_key_holder_data(&key_holder_data)
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
