use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DomainParticipantAsync {
    pub(crate) _inner: dust_dds::dds_async::domain_participant::DomainParticipantAsync,
}

impl DomainParticipantAsync {
    pub(crate) fn new(
        inner: dust_dds::dds_async::domain_participant::DomainParticipantAsync,
    ) -> Self {
        Self { _inner: inner }
    }
}
