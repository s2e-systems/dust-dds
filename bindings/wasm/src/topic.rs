use wasm_bindgen::prelude::*;

use crate::{
    Result, dynamic_type::DynamicType, infrastructure::instance::InstanceHandle,
    result::into_js_error,
};

#[wasm_bindgen]
#[derive(Clone)]
pub struct Topic {
    pub(crate) _inner: dust_dds::dds_async::topic::TopicAsync,
}

impl Topic {
    pub(crate) fn new(inner: dust_dds::dds_async::topic::TopicAsync) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl Topic {
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

    #[wasm_bindgen(js_name = getDynamicType)]
    pub async fn get_dynamic_type(&self) -> Result<DynamicType> {
        let dt = self
            ._inner
            .get_type_support()
            .await
            .map_err(into_js_error)?;
        Ok(DynamicType { inner: dt })
    }
}
