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

    #[wasm_bindgen(js_name = getStatusCondition)]
    pub fn get_status_condition(&self) -> crate::condition::StatusCondition {
        crate::condition::StatusCondition::new(
            self._inner.get_statuscondition(),
            self._inner.get_instance_handle(),
        )
    }

    #[wasm_bindgen(js_name = setListener)]
    pub async fn set_listener(
        &self,
        a_listener: Option<js_sys::Object>,
        mask: Option<js_sys::Array>,
    ) -> Result<()> {
        let listener = a_listener.map(|l| crate::listener::WasmTopicListener {
            listener: crate::listener::SendJsValue(l.into()),
        });
        let status_mask = crate::listener::parse_status_mask(mask)?;
        self._inner
            .set_listener(listener, &status_mask)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }
}
