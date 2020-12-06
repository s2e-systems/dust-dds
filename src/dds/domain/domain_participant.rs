use crate::dds::implementation::rtps_participant::RtpsParticipant;
pub struct DomainParticipant(pub RtpsParticipant);

impl std::ops::Deref for DomainParticipant{
    type Target = RtpsParticipant;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}