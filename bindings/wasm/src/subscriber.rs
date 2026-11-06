use wasm_bindgen::prelude::*;

use crate::infrastructure::instance::InstanceHandle;

#[wasm_bindgen]
pub struct SubscriberAsync {
    pub(crate) _inner: dust_dds::dds_async::subscriber::SubscriberAsync,
}

impl SubscriberAsync {
    pub(crate) fn new(inner: dust_dds::dds_async::subscriber::SubscriberAsync) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl SubscriberAsync {
    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }
}
