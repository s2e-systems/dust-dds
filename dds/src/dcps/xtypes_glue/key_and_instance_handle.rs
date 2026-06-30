use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        dynamic_type::{DynamicData, DynamicDataFactory, DynamicTypeBuilderFactory, TypeKind},
        error::XTypesError,
        serializer::serialize_final_without_header,
    },
};
use alloc::vec::Vec;

pub fn get_instance_handle_from_dynamic_data<'a>(
    dynamic_data: &DynamicData<'a>,
) -> Result<InstanceHandle, XTypesError> {
    let dynamic_type = dynamic_data.r#type();
    let mut key_holder_type_builder =
        DynamicTypeBuilderFactory::create_type(dynamic_data.r#type().descriptor.clone());
    for member_index in 0..dynamic_type.get_member_count() {
        let dynamic_type_member = dynamic_type.get_member_by_index(member_index)?;
        if dynamic_type_member.get_descriptor()?.is_key {
            key_holder_type_builder.add_member(dynamic_type_member.get_descriptor()?.clone())?;
        }
    }
    let key_holder_type = key_holder_type_builder.build();
    
    let mut key_holder_data = DynamicDataFactory::create_data(key_holder_type);
    for key_member_index in 0..key_holder_type.get_member_count() {
        let key_member_id = key_holder_type
            .get_member_by_index(key_member_index)?
            .get_id();
        key_holder_data.set_value(
            key_member_id,
            dynamic_data.get_value(key_member_id)?.clone(),
        );
    }

    let key = if key_holder_data.r#type().get_kind() == TypeKind::STRUCTURE {
        let data = serialize_final_without_header(Vec::new(), &key_holder_data)?;
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
