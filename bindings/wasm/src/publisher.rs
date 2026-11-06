use wasm_bindgen::prelude::*;

use crate::infrastructure::instance::InstanceHandle;

#[wasm_bindgen]
pub struct PublisherAsync {
    pub(crate) _inner: dust_dds::dds_async::publisher::PublisherAsync,
}

impl PublisherAsync {
    pub(crate) fn new(inner: dust_dds::dds_async::publisher::PublisherAsync) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl PublisherAsync {
    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }
}
