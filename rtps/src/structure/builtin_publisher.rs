use std::sync::Mutex;
use std::sync::mpsc;

use crate::types::{GuidPrefix, GUID, EntityId, EntityKind, TopicKind, Locator, ChangeKind};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER;
use crate::behavior::StatelessWriter;
use crate::messages::RtpsSubmessage;
use crate::transport::Transport;

use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

pub struct BuiltinPublisher {
    guid: GUID,
    receiver: mpsc::Receiver<(Vec<Locator>,RtpsSubmessage)>,
    spdp_builtin_participant_writer: Mutex<StatelessWriter>,
}

impl BuiltinPublisher {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        let guid = GUID::new(guid_prefix, EntityId::new([0,0,0], EntityKind::BuiltInWriterGroup));

        let (sender, receiver) = mpsc::channel();

        let spdp_builtin_participant_writer_guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER);
        let mut spdp_builtin_participant_writer_qos = DataWriterQos::default();
        spdp_builtin_participant_writer_qos.reliability.kind = ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        let mut spdp_builtin_participant_writer = StatelessWriter::new(spdp_builtin_participant_writer_guid, TopicKind::WithKey, &spdp_builtin_participant_writer_qos, sender);

        spdp_builtin_participant_writer.reader_locator_add(Locator::new_udpv4(7400, [239,255,0,1]));
        let cc = spdp_builtin_participant_writer.new_change(ChangeKind::Alive, Some(vec![0,0,0,0,1,2,3]), None, [8;16]);
        spdp_builtin_participant_writer.writer_cache().add_change(cc).unwrap();

        Self {
            guid,
            receiver,
            spdp_builtin_participant_writer: Mutex::new(spdp_builtin_participant_writer)
        }
    }

    pub fn receiver(&self) -> &mpsc::Receiver<(Vec<Locator>,RtpsSubmessage)> {
        &self.receiver
    }

    pub fn run(&self, transport: &dyn Transport) {
        self.spdp_builtin_participant_writer.lock().unwrap().unsent_changes_reset();
        self.spdp_builtin_participant_writer.lock().unwrap().run();
    }
}