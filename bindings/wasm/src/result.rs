use wasm_bindgen::JsValue;

pub type Result<T> = core::result::Result<T, JsValue>;
pub type DdsResult<T> = core::result::Result<T, JsValue>;

pub fn into_js_error(error: impl core::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{error:?}"))
}
