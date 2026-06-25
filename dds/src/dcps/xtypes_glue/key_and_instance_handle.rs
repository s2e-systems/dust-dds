use crate::{
    infrastructure::instance::InstanceHandle,
    xtypes::{
        dynamic_type::{DynamicData, TypeKind},
        error::XTypesError,
        serializer::serialize_final_without_header,
    },
};

pub fn get_instance_handle_from_dynamic_data<'a>(
    mut dynamic_data: DynamicData<'a>,
) -> Result<InstanceHandle, XTypesError> {

    let key = if dynamic_data.r#type().get_kind() == TypeKind::STRUCTURE {
        dynamic_data.clear_nonkey_values()?;
        let data = serialize_final_without_header(Vec::new(), &dynamic_data)?;
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
