use std::sync::Mutex;

use crate::types::{GuidPrefix, GUID, EntityId, EntityKind, TopicKind, Locator, ChangeKind};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
use crate::behavior::StatelessWriter;
use crate::messages::message_sender::RtpsMessageSender;

use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

pub struct BuiltinPublisher {
    guid: GUID,
    spdp_builtin_participant_writer: Mutex<StatelessWriter>,
    sender: RtpsMessageSender,
}

impl BuiltinPublisher {
    pub fn new(guid_prefix: GuidPrefix, sender: RtpsMessageSender) -> Self {
        let guid = GUID::new(guid_prefix, EntityId::new([0,0,0], EntityKind::BuiltInWriterGroup));

        let spdp_builtin_participant_writer_guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let mut spdp_builtin_participant_writer_qos = DataWriterQos::default();
        spdp_builtin_participant_writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        let mut spdp_builtin_participant_writer = StatelessWriter::new(spdp_builtin_participant_writer_guid, TopicKind::WithKey, &spdp_builtin_participant_writer_qos);

        spdp_builtin_participant_writer.reader_locator_add(Locator::new_udpv4(7400, [239,255,0,1]));
        let cc = spdp_builtin_participant_writer.new_change(ChangeKind::Alive, Some(vec![0,0,0,0,1,2,3]), None, [8;16]);
        spdp_builtin_participant_writer.writer_cache().add_change(cc).unwrap();

        Self {
            guid,
            spdp_builtin_participant_writer: Mutex::new(spdp_builtin_participant_writer),
            sender,
        }
    }

    pub fn run(&self) {
        self.spdp_builtin_participant_writer.lock().unwrap().unsent_changes_reset();
        self.spdp_builtin_participant_writer.lock().unwrap().run();

        let mut spdp_writer_lock = self.spdp_builtin_participant_writer.lock().unwrap();
        let output_queues = spdp_writer_lock.output_queues();

        for (locator, message) in output_queues {
            let rtps_submessage = message.drain(..).collect();
            self.sender.send(self.guid.prefix(), &locator, rtps_submessage);
        }
    }
}