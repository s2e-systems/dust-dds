use dust_dds::infrastructure::qos::QosKind;
use wasm_bindgen::prelude::*;

use crate::{
    data_writer::DataWriter,
    domain_participant::DomainParticipant,
    dynamic_type::DynamicType,
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos},
    },
    result::{Result, into_js_error},
    topic::Topic,
    type_support::JsDdsData,
};

#[wasm_bindgen]
pub struct Publisher {
    pub(crate) _inner: dust_dds::dds_async::publisher::PublisherAsync,
}

impl Publisher {
    pub(crate) fn new(inner: dust_dds::dds_async::publisher::PublisherAsync) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl Publisher {
    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }

    #[wasm_bindgen(js_name = getParticipant)]
    pub fn get_participant(&self) -> DomainParticipant {
        DomainParticipant::new(self._inner.get_participant())
    }

    #[wasm_bindgen(js_name = createDataWriter)]
    pub async fn create_datawriter(
        &self,
        a_topic: &Topic,
        qos: Option<DataWriterQos>,
        a_listener: Option<js_sys::Object>,
        mask: Option<js_sys::Array>,
    ) -> Result<DataWriter> {
        let dynamic_type = a_topic
            ._inner
            .get_type_support()
            .await
            .map_err(into_js_error)?;

        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };

        let listener = a_listener.map(|l| crate::listener::WasmDataWriterListener {
            listener: crate::listener::SendJsValue(l.into()),
            dynamic_type: DynamicType {
                inner: dynamic_type.clone(),
            },
        });
        let status_mask = crate::listener::parse_status_mask(mask)?;

        let writer = self
            ._inner
            .create_datawriter::<JsDdsData>(&a_topic._inner, qos, listener, &status_mask)
            .await
            .map_err(into_js_error)?;

        Ok(DataWriter::new(
            writer,
            DynamicType {
                inner: dynamic_type,
            },
        ))
    }

    #[wasm_bindgen(js_name = deleteDataWriter)]
    pub async fn delete_datawriter(&self, a_datawriter: &DataWriter) -> Result<()> {
        self._inner
            .delete_datawriter(&a_datawriter._inner)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getDefaultDataWriterQos)]
    pub async fn get_default_datawriter_qos(&self) -> Result<DataWriterQos> {
        let qos = self
            ._inner
            .get_default_datawriter_qos()
            .await
            .map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setDefaultDataWriterQos)]
    pub async fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner
            .set_default_datawriter_qos(qos)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getQos)]
    pub async fn get_qos(&self) -> Result<PublisherQos> {
        let qos = self._inner.get_qos().await.map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setQos)]
    pub async fn set_qos(&self, qos: Option<PublisherQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner.set_qos(qos).await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = beginCoherentChanges)]
    pub async fn begin_coherent_changes(&self) -> Result<()> {
        self._inner
            .begin_coherent_changes()
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = endCoherentChanges)]
    pub async fn end_coherent_changes(&self) -> Result<()> {
        self._inner
            .end_coherent_changes()
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setListener)]
    pub async fn set_listener(
        &self,
        a_listener: Option<js_sys::Object>,
        mask: Option<js_sys::Array>,
    ) -> Result<()> {
        let listener = a_listener.map(|l| crate::listener::WasmPublisherListener {
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
