use crate::result::{Result, into_js_error};
use dust_dds::xtypes::{
    dynamic_type::{DynamicData, DynamicType, TypeKind},
    type_support::{Type, TypeSupport},
};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
pub struct JsDdsData(pub DynamicData<'static>);

impl From<DynamicData<'static>> for JsDdsData {
    fn from(value: DynamicData<'static>) -> Self {
        Self(value)
    }
}

impl From<JsDdsData> for DynamicData<'static> {
    fn from(value: JsDdsData) -> Self {
        value.0
    }
}

impl Type for JsDdsData {
    const TYPE: DynamicType<'static> = <u8 as Type>::TYPE;
}

impl TypeSupport for JsDdsData {
    fn create_sample(src: &mut DynamicData<'static>) -> Option<Self> {
        Some(Self(src.clone()))
    }

    fn create_dynamic_sample(self) -> DynamicData<'static> {
        self.0
    }
}

pub fn convert_js_to_dynamic_data(
    sample: &JsValue,
    dynamic_type: &DynamicType<'static>,
) -> Result<DynamicData<'static>> {
    let mut dynamic_data =
        dust_dds::xtypes::dynamic_type::DynamicDataFactory::create_data(*dynamic_type);

    for member_index in 0..dynamic_type.get_member_count() {
        let member = dynamic_type
            .get_member_by_index(member_index)
            .map_err(into_js_error)?;
        let name = member.get_name();
        let member_descriptor = member.get_descriptor().map_err(into_js_error)?;
        let member_kind = member_descriptor.r#type.get_kind();
        let id = member.get_id();

        let value = js_sys::Reflect::get(sample, &JsValue::from_str(name))
            .map_err(|e| js_sys::Error::new(&format!("Failed to get property '{name}': {e:?}")))?;

        if value.is_undefined() || value.is_null() {
            if member_descriptor.is_optional {
                continue;
            }
        }

        match member_kind {
            TypeKind::NONE => (),
            TypeKind::BOOLEAN => {
                let b = value.as_bool().unwrap_or(false);
                dynamic_data
                    .set_boolean_value(id, b)
                    .map_err(into_js_error)?;
            }
            TypeKind::BYTE => {
                let num = value.as_f64().unwrap_or(0.0) as u8;
                dynamic_data
                    .set_byte_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::INT8 => {
                let num = value.as_f64().unwrap_or(0.0) as i8;
                dynamic_data
                    .set_int8_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::UINT8 => {
                let num = value.as_f64().unwrap_or(0.0) as u8;
                dynamic_data
                    .set_uint8_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::INT16 => {
                let num = value.as_f64().unwrap_or(0.0) as i16;
                dynamic_data
                    .set_int16_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::UINT16 => {
                let num = value.as_f64().unwrap_or(0.0) as u16;
                dynamic_data
                    .set_uint16_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::INT32 => {
                let num = value.as_f64().unwrap_or(0.0) as i32;
                dynamic_data
                    .set_int32_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::UINT32 => {
                let num = value.as_f64().unwrap_or(0.0) as u32;
                dynamic_data
                    .set_uint32_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::INT64 => {
                let num = if value.is_bigint() {
                    let s = js_sys::BigInt::from(value)
                        .to_string(10)
                        .map_err(|e| js_sys::Error::new(&format!("{e:?}")))?;
                    s.as_string()
                        .unwrap_or_default()
                        .parse::<i64>()
                        .unwrap_or(0)
                } else {
                    value.as_f64().unwrap_or(0.0) as i64
                };
                dynamic_data
                    .set_int64_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::UINT64 => {
                let num = if value.is_bigint() {
                    let s = js_sys::BigInt::from(value)
                        .to_string(10)
                        .map_err(|e| js_sys::Error::new(&format!("{e:?}")))?;
                    s.as_string()
                        .unwrap_or_default()
                        .parse::<u64>()
                        .unwrap_or(0)
                } else {
                    value.as_f64().unwrap_or(0.0) as u64
                };
                dynamic_data
                    .set_uint64_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::FLOAT32 => {
                let num = value.as_f64().unwrap_or(0.0) as f32;
                dynamic_data
                    .set_float32_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::FLOAT64 => {
                let num = value.as_f64().unwrap_or(0.0);
                dynamic_data
                    .set_float64_value(id, num)
                    .map_err(into_js_error)?;
            }
            TypeKind::CHAR8 => {
                let ch = value
                    .as_string()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\0');
                dynamic_data
                    .set_char8_value(id, ch)
                    .map_err(into_js_error)?;
            }
            TypeKind::STRING8 => {
                let s = value.as_string().unwrap_or_default();
                dynamic_data
                    .set_string_value(id, s)
                    .map_err(into_js_error)?;
            }
            TypeKind::SEQUENCE => {
                let element_type = member_descriptor
                    .r#type
                    .get_descriptor()
                    .element_type
                    .as_ref()
                    .ok_or_else(|| js_sys::Error::new("Sequence member missing element_type"))?;
                match element_type.get_kind() {
                    TypeKind::UINT8 | TypeKind::BYTE => {
                        let bytes = if value.is_instance_of::<js_sys::Uint8Array>() {
                            js_sys::Uint8Array::new(&value).to_vec()
                        } else if js_sys::Array::is_array(&value) {
                            let arr = js_sys::Array::from(&value);
                            let mut vec = Vec::with_capacity(arr.length() as usize);
                            for i in 0..arr.length() {
                                vec.push(arr.get(i).as_f64().unwrap_or(0.0) as u8);
                            }
                            vec
                        } else {
                            return Err(js_sys::Error::new(&format!(
                                "Field '{name}' expects Uint8Array or array of numbers"
                            ))
                            .into());
                        };
                        dynamic_data
                            .set_uint8_values(id, bytes)
                            .map_err(into_js_error)?;
                    }
                    other => {
                        return Err(js_sys::Error::new(&format!(
                            "Sequence of kind {other:?} is not yet implemented"
                        ))
                        .into());
                    }
                }
            }
            TypeKind::STRUCTURE => {
                let nested_data = convert_js_to_dynamic_data(&value, &member_descriptor.r#type)?;
                dynamic_data
                    .set_complex_value(id, nested_data)
                    .map_err(into_js_error)?;
            }
            other => {
                return Err(js_sys::Error::new(&format!(
                    "TypeKind {other:?} is not supported for field '{name}'"
                ))
                .into());
            }
        }
    }

    Ok(dynamic_data)
}

pub fn convert_dynamic_data_to_js(
    dynamic_data: &DynamicData<'static>,
    dynamic_type: &DynamicType<'static>,
) -> Result<JsValue> {
    let obj = js_sys::Object::new();

    for member_index in 0..dynamic_type.get_member_count() {
        let member = dynamic_type
            .get_member_by_index(member_index)
            .map_err(into_js_error)?;
        let name = member.get_name();
        let member_descriptor = member.get_descriptor().map_err(into_js_error)?;
        let member_kind = member_descriptor.r#type.get_kind();
        let id = member.get_id();

        let js_val: JsValue = match member_kind {
            TypeKind::NONE => JsValue::UNDEFINED,
            TypeKind::BOOLEAN => {
                let v = dynamic_data.get_boolean_value(id).map_err(into_js_error)?;
                JsValue::from_bool(*v)
            }
            TypeKind::BYTE => {
                let v = dynamic_data.get_byte_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::INT8 => {
                let v = dynamic_data.get_int8_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::UINT8 => {
                let v = dynamic_data.get_uint8_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::INT16 => {
                let v = dynamic_data.get_int16_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::UINT16 => {
                let v = dynamic_data.get_uint16_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::INT32 => {
                let v = dynamic_data.get_int32_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::UINT32 => {
                let v = dynamic_data.get_uint32_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::INT64 => {
                let v = dynamic_data.get_int64_value(id).map_err(into_js_error)?;
                js_sys::BigInt::from(*v).into()
            }
            TypeKind::UINT64 => {
                let v = dynamic_data.get_uint64_value(id).map_err(into_js_error)?;
                js_sys::BigInt::from(*v).into()
            }
            TypeKind::FLOAT32 => {
                let v = dynamic_data.get_float32_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v as f64)
            }
            TypeKind::FLOAT64 => {
                let v = dynamic_data.get_float64_value(id).map_err(into_js_error)?;
                JsValue::from_f64(*v)
            }
            TypeKind::CHAR8 => {
                let v = dynamic_data.get_char8_value(id).map_err(into_js_error)?;
                JsValue::from_str(&v.to_string())
            }
            TypeKind::STRING8 => {
                let v = dynamic_data.get_string_value(id).map_err(into_js_error)?;
                JsValue::from_str(v)
            }
            TypeKind::SEQUENCE => {
                let element_type = member_descriptor
                    .r#type
                    .get_descriptor()
                    .element_type
                    .as_ref()
                    .ok_or_else(|| js_sys::Error::new("Sequence member missing element_type"))?;
                match element_type.get_kind() {
                    TypeKind::UINT8 | TypeKind::BYTE => {
                        let bytes = dynamic_data.get_uint8_values(id).map_err(into_js_error)?;
                        let uint8_arr = js_sys::Uint8Array::from(bytes);
                        uint8_arr.into()
                    }
                    other => {
                        return Err(js_sys::Error::new(&format!(
                            "Sequence of kind {other:?} is not yet implemented"
                        ))
                        .into());
                    }
                }
            }
            TypeKind::STRUCTURE => {
                let v = dynamic_data.get_complex_value(id).map_err(into_js_error)?;
                convert_dynamic_data_to_js(v, &member_descriptor.r#type)?
            }
            other => {
                return Err(js_sys::Error::new(&format!(
                    "TypeKind {other:?} is not supported for field '{name}'"
                ))
                .into());
            }
        };

        js_sys::Reflect::set(&obj, &JsValue::from_str(name), &js_val)
            .map_err(|e| js_sys::Error::new(&format!("Failed to set property '{name}': {e:?}")))?;
    }

    Ok(obj.into())
}
