use wasm_bindgen::prelude::*;

use crate::result::Result;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct InstanceHandle(pub(crate) dust_dds::infrastructure::instance::InstanceHandle);

#[wasm_bindgen]
impl InstanceHandle {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Option<Vec<u8>>) -> Result<InstanceHandle> {
        match bytes {
            Some(b) => {
                if b.len() != 16 {
                    return Err(JsValue::from_str("InstanceHandle must be 16 bytes"));
                }
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&b);
                Ok(Self(
                    dust_dds::infrastructure::instance::InstanceHandle::new(arr),
                ))
            }
            None => Ok(Self(dust_dds::infrastructure::instance::HANDLE_NIL)),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> Vec<u8> {
        let arr: [u8; 16] = self.0.into();
        arr.to_vec()
    }

    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let arr: [u8; 16] = self.0.into();
        arr.to_vec()
    }

    #[wasm_bindgen(js_name = equals)]
    pub fn equals(&self, other: &InstanceHandle) -> bool {
        self.0 == other.0
    }
}

impl From<dust_dds::infrastructure::instance::InstanceHandle> for InstanceHandle {
    fn from(value: dust_dds::infrastructure::instance::InstanceHandle) -> Self {
        Self(value)
    }
}

impl From<InstanceHandle> for dust_dds::infrastructure::instance::InstanceHandle {
    fn from(value: InstanceHandle) -> Self {
        value.0
    }
}

impl From<&InstanceHandle> for dust_dds::infrastructure::instance::InstanceHandle {
    fn from(value: &InstanceHandle) -> Self {
        value.0
    }
}

impl From<[u8; 16]> for InstanceHandle {
    fn from(value: [u8; 16]) -> Self {
        Self(dust_dds::infrastructure::instance::InstanceHandle::new(
            value,
        ))
    }
}

impl From<InstanceHandle> for [u8; 16] {
    fn from(value: InstanceHandle) -> Self {
        value.0.into()
    }
}
