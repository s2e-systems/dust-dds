use dust_dds::infrastructure::qos::QosKind;
use wasm_bindgen::prelude::*;

use crate::{
    dynamic_type::DynamicType,
    infrastructure::{
        instance::InstanceHandle, qos::DataWriterQos, status::PublicationMatchedStatus,
    },
    publisher::Publisher,
    result::{Result, into_js_error},
    topic::Topic,
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

    #[wasm_bindgen(js_name = registerInstance)]
    pub async fn register_instance(&self, instance: JsValue) -> Result<Option<InstanceHandle>> {
        let dynamic_data = convert_js_to_dynamic_data(&instance, &self.dynamic_type.inner)?;
        let handle = self
            ._inner
            .register_instance(JsDdsData(dynamic_data))
            .await
            .map_err(into_js_error)?;
        Ok(handle.map(Into::into))
    }

    #[wasm_bindgen(js_name = unregisterInstance)]
    pub async fn unregister_instance(
        &self,
        instance: JsValue,
        handle: Option<InstanceHandle>,
    ) -> Result<()> {
        let dynamic_data = convert_js_to_dynamic_data(&instance, &self.dynamic_type.inner)?;
        self._inner
            .unregister_instance(JsDdsData(dynamic_data), handle.map(Into::into))
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = lookupInstance)]
    pub async fn lookup_instance(&self, instance: JsValue) -> Result<Option<InstanceHandle>> {
        let dynamic_data = convert_js_to_dynamic_data(&instance, &self.dynamic_type.inner)?;
        let handle = self
            ._inner
            .lookup_instance(JsDdsData(dynamic_data))
            .await
            .map_err(into_js_error)?;
        Ok(handle.map(Into::into))
    }

    #[wasm_bindgen(js_name = dispose)]
    pub async fn dispose(&self, data: JsValue, handle: Option<InstanceHandle>) -> Result<()> {
        let dynamic_data = convert_js_to_dynamic_data(&data, &self.dynamic_type.inner)?;
        self._inner
            .dispose(JsDdsData(dynamic_data), handle.map(Into::into))
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = waitForAcknowledgments)]
    pub async fn wait_for_acknowledgments(&self) -> Result<()> {
        self._inner
            .wait_for_acknowledgments()
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getTopic)]
    pub fn get_topic(&self) -> Topic {
        Topic::new(self._inner.get_topic())
    }

    #[wasm_bindgen(js_name = getPublisher)]
    pub fn get_publisher(&self) -> Publisher {
        Publisher::new(self._inner.get_publisher())
    }

    #[wasm_bindgen(js_name = getQos)]
    pub async fn get_qos(&self) -> Result<DataWriterQos> {
        let qos = self._inner.get_qos().await.map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setQos)]
    pub async fn set_qos(&self, qos: Option<DataWriterQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner.set_qos(qos).await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getPublicationMatchedStatus)]
    pub async fn get_publication_matched_status(&self) -> Result<PublicationMatchedStatus> {
        let status = self
            ._inner
            .get_publication_matched_status()
            .await
            .map_err(into_js_error)?;
        Ok(status.into())
    }

    #[wasm_bindgen(js_name = getMatchedSubscriptions)]
    pub async fn get_matched_subscriptions(&self) -> Result<js_sys::Array> {
        let handles = self
            ._inner
            .get_matched_subscriptions()
            .await
            .map_err(into_js_error)?;
        let array = js_sys::Array::new();
        for h in handles {
            array.push(&JsValue::from(InstanceHandle::from(h)));
        }
        Ok(array)
    }

    pub async fn enable(&self) -> Result<()> {
        self._inner.enable().await.map_err(into_js_error)?;
        Ok(())
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
        let listener = a_listener.map(|l| crate::listener::WasmDataWriterListener {
            listener: crate::listener::SendJsValue(l.into()),
            dynamic_type: self.dynamic_type.clone(),
        });
        let status_mask = crate::listener::parse_status_mask(mask)?;
        self._inner
            .set_listener(listener, &status_mask)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }
}
