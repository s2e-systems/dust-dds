use std::sync::{Arc, Mutex};

use crate::reader::Reader;
use crate::structure::{RtpsEntity, RtpsGroup};
use crate::behavior::{types::Duration, stateful_reader::StatefulReader};
use crate::types::ReliabilityKind;
use crate::behavior::stateful_reader::NoOpStatefulReaderListener;

use rust_dds_interface::protocol::{ProtocolSubscriber, ProtocolEntity, ProtocolReader};
use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::qos::DataReaderQos;
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;
use rust_dds_interface::history_cache::HistoryCache;

pub struct Subscriber {
    group: Arc<Mutex<RtpsGroup>>,
}

impl Subscriber {
    pub fn new(group: Arc<Mutex<RtpsGroup>>) -> Self {        
        Self {
            group
        }
    }
}

impl ProtocolEntity for Subscriber {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolSubscriber for Subscriber {
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Box<dyn ProtocolReader> {
        let mut group = self.group.lock().unwrap();
        let reliability_level = match data_reader_qos.reliability.kind {
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = Duration::from_millis(100);
        let reader_cache = HistoryCache::new(data_reader_qos.resource_limits.clone());
        let listener = NoOpStatefulReaderListener;
        let reader = Arc::new(Mutex::new(StatefulReader::new(group.guid(), topic_kind, reliability_level, expects_inline_qos, heartbeat_response_delay, reader_cache, listener)));
        group.mut_endpoints().push(reader.clone());
        Box::new(Reader::new(reader.clone()))
    }
}