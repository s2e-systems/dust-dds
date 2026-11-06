use dust_dds::{
    infrastructure::{configuration::DustDdsConfiguration, qos::QosKind},
    security::plugins::types::DdsSecurityPlugins,
};
use wasm_bindgen::prelude::*;

use crate::{
    domain_participant::DomainParticipant,
    infrastructure::qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    result::{Result, into_js_error},
    runtime::WasmRuntime,
    transport::WasmTransport,
};

#[wasm_bindgen]
pub struct DomainParticipantFactory {
    inner: dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
}

#[wasm_bindgen]
impl DomainParticipantFactory {
    #[wasm_bindgen(constructor)]
    pub fn new(discovery_url: Option<String>) -> Self {
        let configuration = DustDdsConfiguration::default();
        let runtime = WasmRuntime;
        let transport = match discovery_url {
            Some(url) => WasmTransport::with_url(url),
            None => WasmTransport::new(),
        };
        let inner =
            dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync::new(
                configuration,
                runtime,
                transport,
                DdsSecurityPlugins::disabled(),
            );
        Self { inner }
    }

    #[wasm_bindgen(js_name = getInstance)]
    pub fn get_instance(discovery_url: Option<String>) -> Self {
        Self::new(discovery_url)
    }

    #[wasm_bindgen(js_name = createParticipant)]
    pub async fn create_participant(
        &self,
        domain_id: i32,
        qos: Option<DomainParticipantQos>,
        a_listener: Option<js_sys::Object>,
        mask: Option<js_sys::Array>,
    ) -> Result<DomainParticipant> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };

        let listener = a_listener.map(|l| crate::listener::WasmDomainParticipantListener {
            listener: crate::listener::SendJsValue(l.into()),
        });
        let status_mask = crate::listener::parse_status_mask(mask)?;

        let participant = self
            .inner
            .create_participant(domain_id, qos, listener, &status_mask)
            .await
            .map_err(into_js_error)?;

        Ok(DomainParticipant::new(participant))
    }

    #[wasm_bindgen(js_name = deleteParticipant)]
    pub async fn delete_participant(&self, participant: &DomainParticipant) -> Result<()> {
        self.inner
            .delete_participant(&participant._inner)
            .await
            .map_err(into_js_error)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = lookupParticipant)]
    pub async fn lookup_participant(&self, domain_id: i32) -> Result<Option<DomainParticipant>> {
        let participant = self
            .inner
            .lookup_participant(domain_id)
            .await
            .map_err(into_js_error)?;

        Ok(participant.map(DomainParticipant::new))
    }

    #[wasm_bindgen(js_name = setDefaultParticipantQos)]
    pub async fn set_default_participant_qos(
        &self,
        qos: Option<DomainParticipantQos>,
    ) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self.inner
            .set_default_participant_qos(qos)
            .await
            .map_err(into_js_error)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = getDefaultParticipantQos)]
    pub async fn get_default_participant_qos(&self) -> Result<DomainParticipantQos> {
        let qos = self
            .inner
            .get_default_participant_qos()
            .await
            .map_err(into_js_error)?;

        Ok(qos.into())
    }

    #[wasm_bindgen(js_name = setQos)]
    pub async fn set_qos(&self, qos: Option<DomainParticipantFactoryQos>) -> Result<()> {
        let qos = match qos {
            Some(q) => QosKind::Specific(q.into()),
            None => QosKind::Default,
        };
        self.inner.set_qos(qos).await.map_err(into_js_error)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = getQos)]
    pub async fn get_qos(&self) -> Result<DomainParticipantFactoryQos> {
        let qos = self.inner.get_qos().await.map_err(into_js_error)?;

        Ok(qos.into())
    }
}

impl Default for DomainParticipantFactory {
    fn default() -> Self {
        Self::new(None)
    }
}
