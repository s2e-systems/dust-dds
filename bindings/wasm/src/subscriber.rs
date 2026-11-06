use dust_dds::infrastructure::{listener::NO_LISTENER, qos::QosKind, status::NO_STATUS};
use wasm_bindgen::prelude::*;

use crate::{
    data_reader::DataReader,
    domain_participant::DomainParticipant,
    dynamic_type::DynamicType,
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
    },
    result::{Result, into_js_error},
    topic::Topic,
    type_support::JsDdsData,
};

#[wasm_bindgen]
pub struct Subscriber {
    pub(crate) _inner: dust_dds::dds_async::subscriber::SubscriberAsync,
}

impl Subscriber {
    pub(crate) fn new(inner: dust_dds::dds_async::subscriber::SubscriberAsync) -> Self {
        Self { _inner: inner }
    }
}

#[wasm_bindgen]
impl Subscriber {
    #[wasm_bindgen(js_name = getInstanceHandle)]
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self._inner.get_instance_handle().into()
    }

    #[wasm_bindgen(js_name = getParticipant)]
    pub fn get_participant(&self) -> DomainParticipant {
        DomainParticipant::new(self._inner.get_participant())
    }

    #[wasm_bindgen(js_name = createDataReader)]
    pub async fn create_datareader(
        &self,
        a_topic: &Topic,
        qos: Option<DataReaderQos>,
    ) -> Result<DataReader> {
        let dynamic_type = a_topic
            ._inner
            .get_type_support()
            .await
            .map_err(into_js_error)?;

        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };

        let reader = self
            ._inner
            .create_datareader::<JsDdsData>(&a_topic._inner, qos, NO_LISTENER, NO_STATUS)
            .await
            .map_err(into_js_error)?;

        Ok(DataReader::new(
            reader,
            DynamicType {
                inner: dynamic_type,
            },
        ))
    }

    #[wasm_bindgen(js_name = deleteDataReader)]
    pub async fn delete_datareader(&self, a_datareader: &DataReader) -> Result<()> {
        self._inner
            .delete_datareader(&a_datareader._inner)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getDefaultDataReaderQos)]
    pub async fn get_default_datareader_qos(&self) -> Result<DataReaderQos> {
        let qos = self
            ._inner
            .get_default_datareader_qos()
            .await
            .map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setDefaultDataReaderQos)]
    pub async fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner
            .set_default_datareader_qos(qos)
            .await
            .map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getQos)]
    pub async fn get_qos(&self) -> Result<SubscriberQos> {
        let qos = self._inner.get_qos().await.map_err(into_js_error)?;
        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setQos)]
    pub async fn set_qos(&self, qos: Option<SubscriberQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self._inner.set_qos(qos).await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = beginAccess)]
    pub async fn begin_access(&self) -> Result<()> {
        self._inner.begin_access().await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = endAccess)]
    pub async fn end_access(&self) -> Result<()> {
        self._inner.end_access().await.map_err(into_js_error)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = getStatusCondition)]
    pub fn get_status_condition(&self) -> crate::condition::StatusCondition {
        crate::condition::StatusCondition::new(
            self._inner.get_statuscondition(),
            self._inner.get_instance_handle(),
        )
    }
}
