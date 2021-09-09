use rust_dds_api::builtin_topics::ParticipantBuiltinTopicData;
use rust_rtps_pim::{
    behavior::types::Duration, discovery::spdp::participant_proxy::ParticipantProxy,
};

use crate::{
    data_representation_builtin_endpoints::parameter_id_values::PID_PARTICIPANT_LEASE_DURATION,
    data_serialize_deserialize::{
        MappingWriteByteOrdered, ParameterListSerialize, ParameterSerialize,
    },
    dds_type::DdsSerialize,
};

pub struct SpdpDiscoveredParticipantData<'a, L> {
    pub dds_participant_data: ParticipantBuiltinTopicData,
    pub participant_proxy: ParticipantProxy<'a, L>,
    pub lease_duration: Duration,
}

impl<'a, L> DdsSerialize for SpdpDiscoveredParticipantData<'a, L> {
    fn serialize<W: std::io::Write, E: crate::dds_type::Endianness>(
        &self,
        writer: W,
    ) -> rust_dds_api::return_type::DDSResult<()> {
        let parameter_list = ParameterListSerialize(vec![ParameterSerialize::new(
            PID_PARTICIPANT_LEASE_DURATION,
            Box::new([30,30]),
        )]);
        parameter_list.write_ordered::<_, E>(writer).unwrap();
        Ok(())
    }
}
