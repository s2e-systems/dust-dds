use std::sync::Arc;

use rust_dds_interface::types::{ReturnCode, InstanceHandle};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter, ProtocolPublisher};

use crate::types::GUID;

use super::stateful_writer::StatefulWriter;

pub struct RtpsPublisher {
    guid: GUID,
}

impl RtpsPublisher {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid
        }
    }
}

impl ProtocolEntity for RtpsPublisher {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl ProtocolPublisher for RtpsPublisher {
    fn create_writer(&self) -> std::sync::Arc<dyn ProtocolWriter> {
        todo!()
        // Arc::new(StatefulWriter::new(guid, topic_kind, reliability_level, push_mode, heartbeat_period, nack_response_delay, nack_suppression_duration))
    }
}