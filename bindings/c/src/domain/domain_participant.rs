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
