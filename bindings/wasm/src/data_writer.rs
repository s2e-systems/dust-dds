use wasm_bindgen::prelude::*;

use crate::{
    dynamic_type::DynamicType,
    infrastructure::instance::InstanceHandle,
    result::{Result, into_js_error},
    type_support::{JsDdsData, convert_js_to_dynamic_data},
};

#[wasm_bindgen]
pub struct DataWriter {
    pub(crate) _inner: dust_dds::dds_async::data_writer::DataWriterAsync<JsDdsData>,
    pub(crate) dynamic_type: DynamicType,
}

impl DataWriter {
    pub(crate) fn new(
        inner: dust_dds::dds_async::data_writer::DataWriterAsync<JsDdsData>,
        dynamic_type: DynamicType,
    ) -> Self {
        Self {
            _inner: inner,
            dynamic_type,
        }
    }
}

#[wasm_bindgen]
impl DataWriter {
    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }

    #[wasm_bindgen(js_name = write)]
    pub async fn write(&self, data: JsValue, handle: Option<InstanceHandle>) -> Result<()> {
        let dynamic_data = convert_js_to_dynamic_data(&data, &self.dynamic_type.inner)?;
        let handle = handle.map(Into::into);
        self._inner
            .write(JsDdsData(dynamic_data), handle)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getPublicationMatchedStatus)]
    pub async fn get_publication_matched_status(
        &self,
    ) -> Result<crate::infrastructure::status::PublicationMatchedStatus> {
        let status = self
            ._inner
            .get_publication_matched_status()
            .await
            .map_err(into_js_error)?;
        Ok(status.into())
    }
}
