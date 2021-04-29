use rust_dds_api::{builtin_topics::ParticipantBuiltinTopicData, dds_type::DDSType};
use rust_rtps::{behavior::types::Duration, discovery::spdp_data::ParticipantProxy};

pub struct SPDPdiscoveredParticipantData {
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy,
    pub lease_duration: Duration,
}
