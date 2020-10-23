use std::collections::VecDeque;
use std::sync::{Arc,Mutex};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolSubscriber, ProtocolReader};

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::qos::DataReaderQos;
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

use crate::types::{GUID, GuidPrefix, EntityId, EntityKind, Locator};
use crate::types::constants::ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR;
use crate::behavior::StatelessReader;
use crate::transport::Transport;
use crate::messages::RtpsSubmessage;
use crate::messages::message_receiver::{Receiver, RtpsMessageReceiver};

pub struct BuiltinSubscriber {
    guid: GUID,
    spdp_builtin_participant_reader: Mutex<StatelessReader>,
}

impl BuiltinSubscriber {
    pub fn new(guid_prefix: GuidPrefix) -> Self {
        let guid = GUID::new(guid_prefix, EntityId::new([0,0,0], EntityKind::BuiltInReaderGroup));

        let spdp_builtin_participant_reader_guid = GUID::new(guid_prefix, ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR);
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![Locator::new_udpv4(7400, [239,255,0,1])];
        let mut reader_qos = DataReaderQos::default();
        reader_qos.reliability.kind =  ReliabilityQosPolicyKind::BestEffortReliabilityQos;
        let spdp_builtin_participant_reader = Mutex::new(
            StatelessReader::new(
                spdp_builtin_participant_reader_guid,
                TopicKind::WithKey,
                unicast_locator_list,
                multicast_locator_list,
                &reader_qos)
        );

        Self {
            guid,
            spdp_builtin_participant_reader,
        }
    }

    pub fn run(&self) {
        self.spdp_builtin_participant_reader.lock().unwrap().run()
    }
}

impl Receiver for BuiltinSubscriber {
    fn try_push_message(&self, src_locator: Locator, src_guid_prefix: GuidPrefix, submessage: RtpsSubmessage) -> Option<RtpsSubmessage> {
        self.spdp_builtin_participant_reader.lock().unwrap().try_push_message(src_locator, src_guid_prefix, submessage)
    }
}


impl ProtocolEntity for BuiltinSubscriber {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolSubscriber for BuiltinSubscriber {
    fn create_reader(&mut self, _topic_kind: TopicKind, _data_reader_qos: &DataReaderQos) -> Arc<Mutex<dyn ProtocolReader>> {
        todo!()
    }
}