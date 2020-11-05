use std::sync::{Arc, Mutex};
use crate::structure::RtpsGroup;

use rust_dds_interface::protocol::{ProtocolPublisher, ProtocolEntity, ProtocolWriter};
use rust_dds_interface::types::{ReturnCode, InstanceHandle};

pub struct Publisher {
    group: Arc<Mutex<RtpsGroup>>,
}

impl Publisher {
    pub fn new(group: Arc<Mutex<RtpsGroup>>) -> Self {        
        Self {
            group
        }
    }
}

impl ProtocolEntity for Publisher {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolPublisher for Publisher {
    fn create_writer(&mut self, topic_kind: rust_dds_interface::types::TopicKind, data_writer_qos: &rust_dds_interface::qos::DataWriterQos) -> Box<dyn ProtocolWriter> {
        // let guid_prefix = self.guid.prefix();
        // let entity_kind = match topic_kind {
        //     TopicKind::NoKey => EntityKind::UserDefinedReaderNoKey,
        //     TopicKind::WithKey => EntityKind::UserDefinedReaderWithKey,
        // };
        // let entity_key = [self.guid.entity_id().entity_key()[0], self.endpoint_counter as u8, 0];
        // let entity_id = EntityId::new(entity_key, entity_kind);
        // self.endpoint_counter += 1;
        // let writer_guid = GUID::new(guid_prefix, entity_id);

        // let reliability_level = match data_writer_qos.reliability.kind {
        //     ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        //     ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
        // };

        // let resource_limits = HistoryCacheResourceLimits{
        //     max_samples: data_writer_qos.resource_limits.max_samples,
        //     max_instances: data_writer_qos.resource_limits.max_instances,
        //     max_samples_per_instance: data_writer_qos.resource_limits.max_samples_per_instance,
        // };

        // let push_mode = true;
        // let heartbeat_period = Duration::from_millis(500);
        // let nack_response_delay = Duration::from_millis(0);
        // let nack_supression_duration = Duration::from_millis(0);

        // let new_writer = Arc::new(Mutex::new(
        //     StatefulWriter::new(
        //         writer_guid,
        //         topic_kind,
        //         reliability_level,
        //         resource_limits,
        //         push_mode,
        //         heartbeat_period,
        //         nack_response_delay,
        //         nack_supression_duration,
        //     )));
        // self.endpoints.push(new_writer.clone());

        // new_writer
        todo!()
    }
}