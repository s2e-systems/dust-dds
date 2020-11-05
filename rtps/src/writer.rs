use std::sync::{Arc, Mutex};
use crate::behavior::StatefulWriter;

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter};
use rust_dds_interface::types::{ReturnCode, InstanceHandle};

pub struct Writer {
    writer: Arc<Mutex<StatefulWriter>>,
}

impl ProtocolEntity for Writer {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        todo!()
    }
}

impl ProtocolWriter for Writer {
    fn write(&mut self, instance_handle: InstanceHandle, data: rust_dds_interface::types::Data, timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
        todo!()
    }

    fn dispose(&self, instance_handle: InstanceHandle, timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
        todo!()
    }

    fn unregister(&self, instance_handle: InstanceHandle, timestamp: rust_dds_interface::types::Time) -> ReturnCode<()> {
        todo!()
    }

    fn register(&self, instance_handle: InstanceHandle, timestamp: rust_dds_interface::types::Time) -> ReturnCode<Option<InstanceHandle>> {
        todo!()
    }

    fn lookup_instance(&self, instance_handle: InstanceHandle) -> Option<InstanceHandle> {
        todo!()
    }
}