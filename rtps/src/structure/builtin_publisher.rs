use crate::types::{GuidPrefix, GUID, EntityId, EntityKind, TopicKind};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
use crate::behavior::StatelessWriter;
use crate::messages::message_sender::RtpsMessageSender;
use crate::transport::Transport;

use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

pub struct BuiltinPublisher {
    guid: GUID,
    spdp_builtin_participant_writer: StatelessWriter,
}

impl BuiltinPublisher {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        let guid = GUID::new(guid_prefix, EntityId::new([0,0,0], EntityKind::BuiltInWriterGroup));

        let spdp_builtin_participant_writer_guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let mut spdp_builtin_participant_writer_qos = DataWriterQos::default();
        spdp_builtin_participant_writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        let spdp_builtin_participant_writer = StatelessWriter::new(spdp_builtin_participant_writer_guid, TopicKind::WithKey, &spdp_builtin_participant_writer_qos);

        Self {
            guid,
            spdp_builtin_participant_writer
        }
    }

    pub fn send(&self, transport: &dyn Transport) {
        RtpsMessageSender::send(self.guid.prefix(), transport, &[&self.spdp_builtin_participant_writer]);
    }
}