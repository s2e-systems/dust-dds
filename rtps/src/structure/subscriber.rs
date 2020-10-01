use std::sync::Arc;

use rust_dds_interface::types::{ReturnCode, InstanceHandle};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader, ProtocolSubscriber};

use crate::types::GUID;

pub struct RtpsSubscriber{
    guid: GUID,
}

impl RtpsSubscriber {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid
        }
    }
}

impl ProtocolEntity for RtpsSubscriber {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl ProtocolSubscriber for RtpsSubscriber {
    fn create_reader(&self) -> Arc<dyn ProtocolReader> {
        todo!()
    }
}