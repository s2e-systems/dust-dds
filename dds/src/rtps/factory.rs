use crate::transport::{
    factory::TransportParticipantFactory, participant::TransportParticipant, types::GuidPrefix,
};

use super::transport::RtpsTransport;

pub struct RtpsParticipantFactory {
    fragment_size: usize,
}

impl RtpsParticipantFactory {
    pub fn new(fragment_size: usize) -> Self {
        Self { fragment_size }
    }
}

impl Default for RtpsParticipantFactory {
    fn default() -> Self {
        Self {
            fragment_size: 1344,
        }
    }
}

impl TransportParticipantFactory for RtpsParticipantFactory {
    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Box<dyn TransportParticipant> {
        Box::new(
            RtpsTransport::new(guid_prefix, domain_id, None, None, self.fragment_size).unwrap(),
        )
    }
}
