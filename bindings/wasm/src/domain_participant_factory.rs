use dust_dds::{
    infrastructure::{
        configuration::DustDdsConfiguration, listener::NO_LISTENER, qos::QosKind, status::NO_STATUS,
    },
    security::plugins::types::DdsSecurityPlugins,
};
use wasm_bindgen::prelude::*;

use crate::{
    domain_participant::DomainParticipantAsync, runtime::WasmRuntime, transport::WasmTransport,
};

#[wasm_bindgen]
pub struct DomainParticipantFactoryAsync {
    inner: dust_dds::dds_async::domain_participant_factory::DomainParticipantFactoryAsync,
}

#[wasm_bindgen]
impl DomainParticipantFactoryAsync {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let configuration = DustDdsConfiguration::default();
        let runtime = WasmRuntime;
        let transport = WasmTransport::new();
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
    pub fn get_instance() -> Self {
        Self::new()
    }

    pub async fn create_participant(
        &self,
        domain_id: i32,
    ) -> Result<DomainParticipantAsync, JsValue> {
        let participant = self
            .inner
            .create_participant(domain_id, QosKind::Default, NO_LISTENER, NO_STATUS)
            .await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(DomainParticipantAsync::new(participant))
    }

    pub async fn delete_participant(
        &self,
        participant: &DomainParticipantAsync,
    ) -> Result<(), JsValue> {
        self.inner
            .delete_participant(&participant._inner)
            .await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        Ok(())
    }
}

impl Default for DomainParticipantFactoryAsync {
    fn default() -> Self {
        Self::new()
    }
}
