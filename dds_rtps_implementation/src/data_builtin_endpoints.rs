use rust_dds_api::builtin_topics::ParticipantBuiltinTopicData;
use rust_rtps_pim::{
    behavior::types::Duration, discovery::spdp::spdp_discovered_participant_data::ParticipantProxy,
};

pub struct SpdpDiscoveredParticipantData<'a, L> {
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy<'a, L>,
    pub lease_duration: Duration,
}
