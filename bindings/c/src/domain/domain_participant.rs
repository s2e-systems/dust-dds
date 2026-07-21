/// cbindgen:opaque
pub struct DustDdsDomainParticipant(
    pub(crate) dust_dds::domain::domain_participant::DomainParticipant,
);

impl DustDdsDomainParticipant {
    pub fn new(dp: dust_dds::domain::domain_participant::DomainParticipant) -> Self {
        Self(dp)
    }

    pub fn inner(&self) -> &dust_dds::domain::domain_participant::DomainParticipant {
        &self.0
    }
}

/// Frees a DomainParticipant allocated by the C bindings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dust_dds_domain_participant_free(
    participant: *mut DustDdsDomainParticipant,
) {
    if !participant.is_null() {
        unsafe {
            drop(Box::from_raw(participant));
        }
    }
}
