use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        dynamic_type::{
            DynamicData, DynamicDataFactory, DynamicType, DynamicTypeBuilderFactory, TypeKind,
        },
        error::{XTypesError, XTypesResult},
        serializer::serialize_final_without_header,
    },
};
use alloc::vec::Vec;

pub struct KeyHolderType<'a>(DynamicType<'a>);

impl<'a> KeyHolderType<'a> {
    pub fn from_dynamic_type(value: &DynamicType<'a>) -> XTypesResult<Self> {
        let mut key_holder_type_builder =
            DynamicTypeBuilderFactory::create_type(value.descriptor.clone());
        for member_index in 0..value.get_member_count() {
            let dynamic_type_member = value.get_member_by_index(member_index)?;
            if dynamic_type_member.get_descriptor()?.is_key {
                key_holder_type_builder
                    .add_member(dynamic_type_member.get_descriptor()?.clone())?;
            }
        }
        Ok(Self(key_holder_type_builder.build()))
    }
}

pub struct KeyHolderData<'a>(DynamicData<'a>);

impl<'a> KeyHolderData<'a> {
    pub fn from_dynamic_data(value: &DynamicData<'a>) -> XTypesResult<KeyHolderData<'a>> {
        let key_holder_type = KeyHolderType::from_dynamic_type(&value.r#type())?.0;
        let mut key_holder_data = DynamicDataFactory::create_data(key_holder_type);
        for key_member_index in 0..key_holder_type.get_member_count() {
            let key_member_id = key_holder_type
                .get_member_by_index(key_member_index)?
                .get_id();
            key_holder_data.set_value(key_member_id, value.get_value(key_member_id)?.clone());
        }
        Ok(Self(key_holder_data))
    }

    pub fn as_dynamic_data(&self) -> &DynamicData<'a> {
        &self.0
    }
}

pub fn get_instance_handle_from_key_holder_data<'a>(
    key_holder_data: &KeyHolderData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let key = if key_holder_data.0.r#type().get_kind() == TypeKind::STRUCTURE {
        let data = serialize_final_without_header(Vec::new(), &key_holder_data.0)?;
        if data.len() <= 16 {
            let mut key = [0; 16];
            key[0..data.len()].copy_from_slice(&data);
            key
        } else {
            md5::compute(data).into()
        }
    } else {
        [0; 16]
    };
    Ok(InstanceHandle::new(key))
}

pub fn get_instance_handle_from_dynamic_data<'a>(
    value: &DynamicData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let key_holder_data = KeyHolderData::from_dynamic_data(value)?;
    get_instance_handle_from_key_holder_data(&key_holder_data)
}
