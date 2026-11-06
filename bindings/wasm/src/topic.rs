use wasm_bindgen::prelude::*;

use crate::infrastructure::instance::InstanceHandle;

#[wasm_bindgen]
pub struct TopicAsync {
    pub(crate) _inner: dust_dds::dds_async::topic::TopicAsync,
}

impl TopicAsync {
    pub(crate) fn new(inner: dust_dds::dds_async::topic::TopicAsync) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl TopicAsync {
    #[wasm_bindgen(js_name = getName)]
    pub fn get_name(&self) -> String {
        use dust_dds::dds_async::topic_description::TopicDescriptionAsync;
        self._inner.get_name()
    }

    #[wasm_bindgen(js_name = getTypeName)]
    pub fn get_type_name(&self) -> String {
        use dust_dds::dds_async::topic_description::TopicDescriptionAsync;
        self._inner.get_type_name()
    }

    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }
}
