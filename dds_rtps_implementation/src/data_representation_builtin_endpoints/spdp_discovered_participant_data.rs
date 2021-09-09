use rust_dds_api::builtin_topics::ParticipantBuiltinTopicData;
use rust_rtps_pim::{
    behavior::types::Duration, discovery::spdp::participant_proxy::ParticipantProxy,
};

use crate::dds_type::DdsSerialize;

pub struct SpdpDiscoveredParticipantData<'a, L> {
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy<'a, L>,
    pub lease_duration: Duration,
}

impl<'a, L> DdsSerialize for SpdpDiscoveredParticipantData<'a, L> {
    fn serialize<W: std::io::Write, E: crate::dds_type::Endianness>(
        &self,
        mut writer: W,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        writer.write(&E::REPRESENTATION_IDENTIFIER).unwrap();
        writer.write(&E::REPRESENTATION_OPTIONS).unwrap();
        Ok(())
    }
}
