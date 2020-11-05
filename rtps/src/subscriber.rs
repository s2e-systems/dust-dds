use std::sync::{Arc, Mutex};

use crate::reader::Reader;
use crate::structure::{RtpsEntity, RtpsGroup, HistoryCacheResourceLimits};
use crate::behavior::{types::Duration, stateful_reader::StatefulReader};
use crate::types::{GUID, EntityId, EntityKind, TopicKind, ReliabilityKind,};
use crate::behavior::stateful_reader::NoOpStatefulReaderListener;

use rust_dds_interface::protocol::{ProtocolSubscriber, ProtocolEntity, ProtocolReader};
use rust_dds_interface::types::{ReturnCode, InstanceHandle};
use rust_dds_interface::qos::DataReaderQos;
use rust_dds_interface::qos_policy::ReliabilityQosPolicyKind;

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
        let resource_limits = HistoryCacheResourceLimits{
            max_samples: data_reader_qos.resource_limits.max_samples,
            max_instances: data_reader_qos.resource_limits.max_instances,
            max_samples_per_instance: data_reader_qos.resource_limits.max_samples_per_instance,
        };
        let listener = NoOpStatefulReaderListener;
        let reader = Arc::new(Mutex::new(StatefulReader::new(group.guid(), topic_kind, reliability_level, expects_inline_qos, heartbeat_response_delay, resource_limits, listener)));
        group.mut_endpoints().push(reader.clone());
        Box::new(Reader::new(reader.clone()))
    }
}