use rust_dds_api::{builtin_topics::ParticipantBuiltinTopicData, dds_type::DDSType};
use rust_rtps::{behavior::types::Duration, discovery::spdp_data::ParticipantProxy};

pub struct SPDPdiscoveredParticipantData {
    dds_participant_data: ParticipantBuiltinTopicData,
    participant_proxy: ParticipantProxy,
    lease_duration: Duration,
}

impl DDSType for SPDPdiscoveredParticipantData {
    fn type_name() -> &'static str {
        todo!()
    }

    fn has_key() -> bool {
        todo!()
    }

    fn key(&self) -> Vec<u8> {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        vec![1,2,3,4]
    }

    fn deserialize(_data: Vec<u8>) -> Self {
        todo!()
    }
}
